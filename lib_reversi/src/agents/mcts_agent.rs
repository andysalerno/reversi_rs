mod tree_search;

use crate::util;
use lib_boardgame::game_primitives::GameResult;
use lib_boardgame::game_primitives::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::rc_tree::RcNode;
use monte_carlo_tree::Node;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::time::Instant;
use tree_search::{Data, MctsData};

pub type MCTSRcAgent<TState> = MCTSAgent<TState, RcNode<MctsData<TState>>>;

pub struct MCTSAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    tree_root: RefCell<TNode>,
    color: PlayerColor,
}

impl<TState, TNode> MCTSAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    pub fn new(color: PlayerColor) -> Self {
        let initial_state = TState::initial_state();
        let data = MctsData::new(&initial_state, 0, 0, None);
        MCTSAgent {
            tree_root: RefCell::new(TNode::new_root(data)),
            color,
        }
    }

    fn backprop(&self, node: &TNode, result: GameResult) {
        let data = node.data();
        data.increment_plays();

        let incr_wins = ((result == GameResult::BlackWins && self.color == PlayerColor::Black)
            || (result == GameResult::WhiteWins && self.color == PlayerColor::White));

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
}

impl<TState, TNode> GameAgent<TState> for MCTSAgent<TState, TNode>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let turn_root = TNode::new_root(MctsData::new(state, 0, 0, None));

        let now = Instant::now();

        const total_sims: u128 = 1000;
        for _ in 0..total_sims {
            // select
            let child_borrowable = select_to_leaf::<TNode, TState>(&turn_root);
            let selected = child_borrowable.borrow();

            // expand
            let newly_expanded_children = expand(selected).into_iter().collect::<Vec<_>>();

            // after we expand, all children are new, so the first one is as good as any
            let sim_node = newly_expanded_children
                .get(0)
                .expect("there must have been children after expanding.");

            // simulate
            let sim_result = simulate(sim_node);

            // backprop
            self.backprop(selected, sim_result);
        }

        let elapsed_micros = now.elapsed().as_micros();
        println!(
            "{} sims total. {} sims/sec.",
            total_sims,
            (total_sims / elapsed_micros) * 1_000_000
        );

        let state_children = turn_root.children();
        let max_child = state_children
            .into_iter()
            .max_by_key(|c| c.data().plays())
            .unwrap();

        let max_action = max_child.data().action().unwrap();

        println!(
            "Plays: {} Wins: {}",
            max_child.data().plays(),
            max_child.data().wins()
        );

        max_action
    }
}

fn select_child<TNode, TState>(nodes: TNode::ChildrenIter) -> Option<TNode::Borrowable>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let selected = nodes
        .into_iter()
        .max_by(|a, b| score_node(a).partial_cmp(&score_node(b)).unwrap());

    match selected {
        Some(n) => Some(n.make_borrowable()),
        None => None,
    }
}

fn select_to_leaf<TNode, TState>(root: &TNode) -> TNode::Borrowable
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.make_borrowable();

    loop {
        let selected_child: Option<TNode::Borrowable> =
            select_child::<TNode, TState>(cur_node.borrow().children());

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

fn expand<TNode, TState>(node: &TNode) -> TNode::ChildrenIter
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let children_iter = node.children().into_iter();
    let children: Vec<_> = children_iter.collect();
    if !children.is_empty() {
        panic!("wtf? we expanded a node that was already expanded.");
    }

    let state = node.data().state();
    let player_turn = state.current_player_turn();
    let legal_actions = state.legal_moves(player_turn);

    for action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = MctsData::new(&resulting_state, 0, 0, Some(action));
        let _child_node = node.new_child(data);
    }

    node.children()
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

/// Given a node, score it in such a way that encourages
/// both exploration and exploitation of the state space.
fn score_node<TNode, TState>(node: &TNode) -> f32
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let plays = node.data().plays() as f32;

    if plays == 0f32 {
        return std::f32::MAX;
    }

    let wins = node.data().wins() as f32;
    let parent_plays = node.parent().map_or(0, |p| p.borrow().data().plays()) as f32;
    let bias = 2 as f32;

    (wins / plays) + (bias * f32::sqrt(f32::ln(parent_plays) / plays))
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
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);
        let tree_root = make_node(data.clone());

        backprop(&tree_root, GameResult::BlackWins);

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
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(4, expanded_children.len());
    }

    #[test]
    fn backprop_works_several_nodes() {
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());

        backprop(&child_level_3, GameResult::BlackWins);

        assert_eq!(1, child_level_3.data().plays());
        assert_eq!(1, child_level_2.data().plays());
        assert_eq!(1, child_level_1.data().plays());
        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, child_level_4.data().plays());
    }

    #[test]
    fn select_child_works() {
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = RcNode::new_root(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4b = child_level_3.new_child(data.clone());

        backprop(&child_level_3, GameResult::BlackWins);
        backprop(&child_level_4, GameResult::BlackWins);
        backprop(&child_level_4, GameResult::BlackWins);
        backprop(&child_level_4b, GameResult::BlackWins);

        let selected_borrow =
            select_child::<RcNode<MctsData<ReversiState>>, ReversiState>(child_level_3.children())
                .expect("the child should have been selected.");

        let selected: &RcNode<MctsData<ReversiState>> = selected_borrow.borrow();

        assert_eq!(1, selected.data().plays());
    }

    #[test]
    fn select_to_leaf_works() {
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = RcNode::new_root(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_3 = child_level_2.new_child(data.clone());
        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4b = child_level_3.new_child(data.clone());

        backprop(&child_level_3, GameResult::BlackWins);
        backprop(&child_level_4, GameResult::BlackWins);
        backprop(&child_level_4, GameResult::BlackWins);
        backprop(&child_level_4b, GameResult::BlackWins);

        let leaf_borrow = select_to_leaf(&tree_root);

        let leaf: &RcNode<MctsData<ReversiState>> = leaf_borrow.borrow();

        assert_eq!(1, leaf.data().plays());
    }

    #[test]
    fn score_node_works() {
        let data = MctsData::new(&ReversiState::new(), 0, 0, None);

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.new_child(data.clone());
        let child_b = tree_root.new_child(data.clone());
        let child_c = tree_root.new_child(data.clone());
        let child_d = tree_root.new_child(data.clone());

        // "visit" each child a different amount of times
        backprop(&child_a, GameResult::BlackWins);
        backprop(&child_a, GameResult::BlackWins);
        backprop(&child_a, GameResult::BlackWins);

        backprop(&child_b, GameResult::BlackWins);
        backprop(&child_b, GameResult::BlackWins);

        backprop(&child_c, GameResult::BlackWins);

        assert_eq!(2.545643f32, score_node(&child_a));
        assert_eq!(2.8930185f32, score_node(&child_b));
        assert_eq!(3.6771324f32, score_node(&child_c));
        assert_eq!(
            340282350000000000000000000000000000000f32,
            score_node(&child_d)
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
