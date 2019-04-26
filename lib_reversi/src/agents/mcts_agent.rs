mod tree_search;

use crate::util;
use lib_boardgame::game_primitives::GameResult;
use lib_boardgame::game_primitives::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::rc_tree::RcNode;
use monte_carlo_tree::Node;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::time::Instant;
use tree_search::{Data, MctsData};

pub type MCTSRcAgent<TState> = MCTSAgent<TState, RcNode<MctsData<TState>>>;

pub struct MCTSAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    color: PlayerColor,
    _phantom_a: PhantomData<TState>,
    _phantom_b: PhantomData<TNode>,
}

impl<TState, TNode> MCTSAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    pub fn new(color: PlayerColor) -> Self {
        MCTSAgent {
            color,
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        }
    }

    fn backprop(&self, node: &TNode, result: GameResult) {
        let data = node.data();
        data.increment_plays();

        let incr_wins = (result == GameResult::BlackWins && self.color == PlayerColor::Black)
            || (result == GameResult::WhiteWins && self.color == PlayerColor::White);

        if incr_wins {
            data.increment_wins();
        }

        let mut parent_node = node.parent();

        while let Some(p) = parent_node {
            let parent = p.borrow();
            let data = parent.data();
            data.increment_plays();

            if incr_wins {
                data.increment_wins();
            }
            parent_node = parent.parent();
        }
    }

    fn select_child(&self, root: &TNode::Borrowable) -> Option<TNode::Borrowable> {
        let root_borrowed = root.borrow();

        let child_nodes = root_borrowed.children();

        let selected = child_nodes
            .into_iter()
            .max_by(|a, b| self.score_node(a).partial_cmp(&self.score_node(b)).unwrap());

        match selected {
            Some(n) => Some(n.make_borrowable()),
            None => None,
        }
    }

    fn select_to_leaf(&self, root: &TNode) -> TNode::Borrowable {
        let mut cur_node = root.make_borrowable();

        loop {
            let selected_child: Option<TNode::Borrowable> = self.select_child(&cur_node);

            if selected_child.is_none() {
                return cur_node;
            }

            cur_node = selected_child.unwrap();
        }
    }

    /// Given a node, score it in such a way that encourages
    /// both exploration and exploitation of the state space.
    fn score_node(&self, node: &TNode) -> f32 {
        let plays = node.data().plays() as f32;

        if plays == 0f32 {
            return std::f32::MAX;
        }

        let wins = node.data().wins() as f32;
        let parent_plays = node.parent().map_or(0, |p| p.borrow().data().plays()) as f32;
        let bias = 2 as f32;

        (wins / plays) + (bias * f32::sqrt(f32::ln(parent_plays) / plays))
    }
}

impl<TState, TNode> GameAgent<TState> for MCTSAgent<TState, TNode>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let turn_root = TNode::new_root(MctsData::new(state, 0, 0, None));

        let now = Instant::now();

        const TOTAL_SIMS: u128 = 1000;
        for _ in 0..TOTAL_SIMS {
            // select
            let child_borrowable = self.select_to_leaf(&turn_root);
            let selected = child_borrowable.borrow();

            // expand
            let expanded = expand(selected);

            if expanded.is_none() {
                // there was nothing left to expand down this path
                continue;
            }

            let newly_expanded_children = expanded.unwrap().into_iter().collect::<Vec<_>>();

            // after we expand, all children are new, so the first one is as good as any
            let sim_node = util::random_pick(&newly_expanded_children);
                // .expect("there must have been children after expanding.");

            // simulate
            let sim_result = simulate(sim_node);

            // backprop
            self.backprop(selected, sim_result);
        }

        let elapsed_micros = now.elapsed().as_micros();
        println!(
            "{} sims total. {:.2} sims/sec.",
            TOTAL_SIMS,
            (TOTAL_SIMS as f64 / elapsed_micros as f64) * 1_000_000f64
        );

        let state_children = turn_root.children();
        let max_child = state_children
            .into_iter()
            .max_by_key(|c| c.data().plays())
            .unwrap();

        let max_action = max_child.data().action().unwrap();

        let plays = max_child.data().plays();
        let wins = max_child.data().wins();
        println!(
            "Plays: {} Wins: {} ({:.2})",
            plays,
            wins,
            wins as f32 / plays as f32,
        );

        max_action
    }
}

fn expand<TNode, TState>(node: &TNode) -> Option<TNode::ChildrenIter>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    // todo: unnecessary optimization here?
    let children_iter = node.children().into_iter();
    let children: Vec<_> = children_iter.collect();
    if !children.is_empty() {
        panic!("wtf? we expanded a node that was already expanded.");
    }

    let state = node.data().state();
    if state.is_game_over() {
        // if the game is over, we have nothing to expand
        return None;
    }

    let player_turn = state.current_player_turn();
    let legal_actions = state.legal_moves(player_turn);

    for action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = MctsData::new(&resulting_state, 0, 0, Some(action));
        let _child_node = node.new_child(data);
    }

    Some(node.children())
}

fn simulate<TNode, TState>(node: &TNode) -> GameResult
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut state = node.data().state().clone();

    loop {
        if state.is_game_over() {
            return state
                .game_result()
                .expect("There must be a game result, since the game is confirmed to be over.");
        }

        let player = state.current_player_turn();
        let legal_moves = state.legal_moves(player);
        let random_action = util::random_choice(&legal_moves);

        state.apply_move(random_action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reversi_gamestate::ReversiState;
    use monte_carlo_tree::rc_tree::RcNode;

    // to ensure clean testing, we get our nodes from this function which gives an anonymous 'impl' type.
    // this way, we know we can behave generically over different impls of the same trait.
    fn make_node(data: MctsData<ReversiState>) -> impl Node<Data = MctsData<ReversiState>> {
        RcNode::new_root(data)
    }

    #[test]
    fn new_child_works() {
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);
        let tree_root = make_node(data.clone());
        let child = tree_root.new_child(data.clone());

        assert_eq!(1, tree_root.children().into_iter().count());
        assert!(child.parent().is_some());
        assert!(tree_root.parent().is_none());
    }

    #[test]
    fn backprop_works_one_node() {
        let agent = MCTSAgent::new(PlayerColor::White);

        let data = MctsData::new(&ReversiState::new(), 0, 0, None);
        let tree_root = make_node(data.clone());

        agent.backprop(&tree_root, GameResult::BlackWins);

        assert_eq!(1, tree_root.data().plays());
    }

    #[test]
    fn expand_works() {
        let mut initial_state = ReversiState::new();

        // default Reversi initial configuration
        initial_state.initialize_board();

        let data = MctsData::new(&initial_state, 0, 0, None);
        let tree_root = RcNode::new_root(data);

        let expanded_children = expand::<RcNode<MctsData<ReversiState>>, ReversiState>(&tree_root)
            .expect("must have children")
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(4, expanded_children.len());
    }

    #[test]
    fn backprop_works_several_nodes() {
        let agent = MCTSAgent::new(PlayerColor::White);

        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());

        agent.backprop(&child_level_3, GameResult::BlackWins);

        assert_eq!(1, child_level_3.data().plays());
        assert_eq!(1, child_level_2.data().plays());
        assert_eq!(1, child_level_1.data().plays());
        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, child_level_4.data().plays());
    }

    #[test]
    fn select_child_works() {
        let agent = MCTSAgent::new(PlayerColor::White);
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = RcNode::new_root(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4b = child_level_3.new_child(data.clone());

        agent.backprop(&child_level_3, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4b, GameResult::BlackWins);

        let selected_borrow = agent
            .select_child(&child_level_3)
            .expect("the child should have been selected.");

        let selected: &RcNode<MctsData<ReversiState>> = selected_borrow.borrow();

        assert_eq!(1, selected.data().plays());
    }

    #[test]
    fn select_to_leaf_works() {
        let agent = MCTSAgent::new(PlayerColor::White);
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = RcNode::new_root(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4b = child_level_3.new_child(data.clone());

        agent.backprop(&child_level_3, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4, GameResult::BlackWins);
        agent.backprop(&child_level_4b, GameResult::BlackWins);
        agent.backprop(&child_level_4b, GameResult::BlackWins);

        let leaf_borrow = agent.select_to_leaf(&tree_root);

        let leaf: &RcNode<MctsData<ReversiState>> = leaf_borrow.borrow();

        assert_eq!(2, leaf.data().plays());
    }

    #[test]
    fn score_node_works() {
        let agent = MCTSAgent::new(PlayerColor::White);
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.new_child(data.clone());
        let child_b = tree_root.new_child(data.clone());
        let child_c = tree_root.new_child(data.clone());
        let child_d = tree_root.new_child(data.clone());

        // "visit" each child a different amount of times
        agent.backprop(&child_a, GameResult::BlackWins);
        agent.backprop(&child_a, GameResult::BlackWins);
        agent.backprop(&child_a, GameResult::BlackWins);

        agent.backprop(&child_b, GameResult::BlackWins);
        agent.backprop(&child_b, GameResult::BlackWins);

        agent.backprop(&child_c, GameResult::BlackWins);

        assert_eq!(1.5456431, agent.score_node(&child_a));
        assert_eq!(1.8930185, agent.score_node(&child_b));
        assert_eq!(2.6771324, agent.score_node(&child_c));
        assert_eq!(
            340282350000000000000000000000000000000f32,
            agent.score_node(&child_d)
        );
    }

    #[test]
    fn simulate_runs_to_completion_and_terminates() {
        let mut initial_state = ReversiState::new();
        initial_state.initialize_board();
        let data = MctsData::new(&initial_state, 0, 0, None);

        let tree_root = make_node(data.clone());

        let _sim_result = simulate(&tree_root);
    }
}
