mod tree_search;

use crate::util;
use lib_boardgame::GameResult;
use lib_boardgame::{GameAgent, GameState, PlayerColor};
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

    // todo: the fact that I require these lines must mean something is wrong...
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

    // TODO: this can become a function, not a method,
    // by passing in the opinionated game result (i.e. it knows if "we" won)
    fn backprop_sim_result(&self, node: &TNode, result: GameResult) {
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

    /// Repeatedly select nodes down the tree until a leaf is reached.
    /// If the given root node is already a leaf,
    /// or is saturated, it is returned.
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

    /// For all children of the given node, assign each one a score,
    /// and return the child with the highest score (ties broken by the first)
    /// or None if there are no children (or if every child is already saturated).
    fn select_child(&self, root: &TNode::Borrowable) -> Option<TNode::Borrowable> {
        let child_nodes = root.borrow().children();

        let selected = child_nodes
            .into_iter()
            .filter(|n| !n.borrow().data().is_saturated())
            .max_by(|a, b| {
                self.score_node(a.borrow())
                    .partial_cmp(&self.score_node(b.borrow()))
                    .unwrap()
            });

        selected
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
            // if our previous work has saturated the tree, we can break early,
            // since we have visited every single outcome already
            if turn_root.borrow().data().is_saturated() {
                break;
            }

            // select the leaf node that we will expand
            let leaf = self.select_to_leaf(turn_root.borrow());
            let leaf = leaf.borrow();

            // expand (create the child nodes for the selected leaf node)
            let expanded_children = expand(leaf);

            if expanded_children.is_none() {
                // there was nothing left to expand down this path
                // TODO: remove this sanity check
                assert!(leaf.children().into_iter().collect::<Vec<_>>().len() == 0);

                // we now know we have selected a saturated node that can't be expanded,
                // so we can update the parent's saturated child count.
                backprop_saturation(leaf);

                // // did backpropagation result in our root node
                // // becoming saturated? if so, we've exhausted
                // // the entire remaining state space, so there's
                // // no more work left to do.
                // if turn_root.borrow().data().is_saturated() && self.color == PlayerColor::Black {
                //     println!("Completely saturated!");
                //     break;
                // } else {
                //     continue;
                // }
            }

            let newly_expanded_children =
                expanded_children.unwrap().into_iter().collect::<Vec<_>>();

            let sim_node = util::random_pick(&newly_expanded_children);

            // simulate
            let sim_result = simulate(sim_node.borrow());

            // backprop
            self.backprop_sim_result(leaf, sim_result);
        }

        let elapsed_micros = now.elapsed().as_micros();
        println!(
            "{} sims total. {:.2} sims/sec.",
            TOTAL_SIMS,
            (TOTAL_SIMS as f64 / elapsed_micros as f64) * 1_000_000f64
        );

        let state_children = turn_root.borrow().children();
        let max_child = state_children
            .into_iter()
            .max_by_key(|c| c.borrow().data().plays())
            .unwrap();

        let max_action = max_child.borrow().data().action().unwrap();

        let plays = max_child.borrow().data().plays();
        let wins = max_child.borrow().data().wins();
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
    node.data().mark_expanded();

    // todo: unnecessary optimization here?
    {
        let children = node.children().into_iter().collect::<Vec<_>>();
        assert!(children.is_empty());
    }

    let state = node.data().state();
    if state.is_game_over() {
        // if the game is over, we have nothing to expand
        return None;
    }

    let player_turn = state.current_player_turn();
    let legal_actions = state.legal_moves(player_turn);

    // Now that we've expanded this node, update it to
    // inform it how many children it has.
    node.data().set_children_count(legal_actions.len());

    // create a new child node for every available action->state transition
    for action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = MctsData::new(&resulting_state, 0, 0, Some(action));
        let _child_node = node.new_child(data);
    }

    Some(node.children())
}

/// Increment this node's count of saturated children.
/// If doing so results in this node itself becoming saturated,
/// follow the same operation for its parent.
fn backprop_saturation<TNode, TState>(node: &TNode)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut parent_node = node.parent();

    while let Some(p) = parent_node {
        let parent = p.borrow();
        let data = parent.data();
        data.increment_saturated_children_count();

        if !data.is_saturated() {
            // we incremented but we're still not saturated,
            // so don't keep going.
            return;
        }

        parent_node = parent.parent();
    }
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
    use monte_carlo_tree::rc_tree::RcNode;

    // commenting out tests for now, need to replace ReversiState with an impl for testing 

    // to ensure clean testing, we get our nodes from this function which gives an anonymous 'impl' type.
    // this way, we know we can behave generically over different impls of the same trait.
    // fn make_node(data: MctsData<ReversiState>) -> impl Node<Data = MctsData<ReversiState>> {
    //     RcNode::new_root(data)
    // }

    // #[test]
    // fn new_child_works() {
    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);
    //     let tree_root = make_node(data.clone());
    //     let child = tree_root.new_child(data.clone());

    //     assert_eq!(1, tree_root.children().into_iter().count());
    //     assert!(child.borrow().parent().is_some());
    //     assert!(tree_root.parent().is_none());
    // }

    // #[test]
    // fn backprop_works_one_node() {
    //     let agent = MCTSAgent::new(PlayerColor::White);

    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);
    //     let tree_root = make_node(data.clone());

    //     agent.backprop_sim_result(&tree_root, GameResult::BlackWins);

    //     assert_eq!(1, tree_root.data().plays());
    // }

    // #[test]
    // fn expand_works() {
    //     let mut initial_state = ReversiState::new();

    //     // default Reversi initial configuration
    //     initial_state.initialize_board();

    //     let data = MctsData::new(&initial_state, 0, 0, None);
    //     let tree_root = RcNode::new_root(data);

    //     let expanded_children = expand::<RcNode<MctsData<ReversiState>>, ReversiState>(&tree_root)
    //         .expect("must have children")
    //         .into_iter()
    //         .collect::<Vec<_>>();

    //     assert_eq!(4, expanded_children.len());
    // }

    // #[test]
    // fn backprop_works_several_nodes() {
    //     let agent = MCTSAgent::new(PlayerColor::White);

    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);

    //     let tree_root = make_node(data.clone());
    //     let child_level_1 = tree_root.new_child(data.clone());
    //     let child_level_2 = child_level_1.borrow().new_child(data.clone());
    //     let child_level_3 = child_level_2.borrow().new_child(data.clone());
    //     let child_level_4 = child_level_3.borrow().new_child(data.clone());

    //     agent.backprop_sim_result(child_level_3.borrow(), GameResult::BlackWins);

    //     assert_eq!(1, child_level_3.borrow().data().plays());
    //     assert_eq!(1, child_level_2.borrow().data().plays());
    //     assert_eq!(1, child_level_1.borrow().data().plays());
    //     assert_eq!(1, tree_root.data().plays());
    //     assert_eq!(0, child_level_4.borrow().data().plays());
    // }

    // #[test]
    // fn select_child_works() {
    //     let agent = MCTSAgent::new(PlayerColor::White);
    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);

    //     let tree_root = RcNode::new_root(data.clone());
    //     let child_level_1 = tree_root.new_child(data.clone());
    //     let child_level_2 = child_level_1.new_child(data.clone());
    //     let child_level_3 = child_level_2.new_child(data.clone());
    //     let child_level_4 = child_level_3.new_child(data.clone());
    //     let child_level_4b = child_level_3.new_child(data.clone());

    //     child_level_1.data().set_children_count(1);
    //     child_level_2.data().set_children_count(1);
    //     child_level_3.data().set_children_count(2);
    //     child_level_4.data().set_children_count(1);
    //     child_level_4b.data().set_children_count(1);

    //     agent.backprop_sim_result(&child_level_3, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4b, GameResult::BlackWins);

    //     assert!(!child_level_3.data().is_saturated());

    //     let selected_borrow = agent
    //         .select_child(&child_level_3)
    //         .expect("the child should have been selected.");

    //     let selected: &RcNode<MctsData<ReversiState>> = selected_borrow.borrow();

    //     assert_eq!(1, selected.data().plays());
    // }

    // #[test]
    // fn select_to_leaf_works() {
    //     let agent = MCTSAgent::new(PlayerColor::White);
    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);

    //     let tree_root = RcNode::new_root(data.clone());
    //     let child_level_1 = tree_root.new_child(data.clone());
    //     let child_level_2 = child_level_1.new_child(data.clone());
    //     let child_level_3 = child_level_2.new_child(data.clone());
    //     let child_level_4 = child_level_3.new_child(data.clone());
    //     let child_level_4b = child_level_3.new_child(data.clone());

    //     tree_root.data().set_children_count(1);
    //     child_level_1.data().set_children_count(1);
    //     child_level_2.data().set_children_count(1);
    //     child_level_3.data().set_children_count(2);
    //     child_level_4.data().set_children_count(2);
    //     child_level_4b.data().set_children_count(2);

    //     agent.backprop_sim_result(&child_level_3, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4b, GameResult::BlackWins);
    //     agent.backprop_sim_result(&child_level_4b, GameResult::BlackWins);

    //     let leaf = agent.select_to_leaf(&tree_root);

    //     let leaf: &RcNode<MctsData<ReversiState>> = leaf.borrow();

    //     assert_eq!(2, leaf.data().plays());
    // }

    // #[test]
    // fn select_to_leaf_when_already_leaf_returns_self() {
    //     let agent = MCTSAgent::new(PlayerColor::White);
    //     let data = MctsData::new(&ReversiState::new(), 10, 10, None);

    //     let tree_root = RcNode::new_root(data.clone());

    //     let leaf = agent.select_to_leaf(&tree_root);

    //     assert_eq!(10, leaf.data().plays());
    //     assert_eq!(10, leaf.data().wins());
    // }

    // #[test]
    // fn backprop_saturation_becomes_saturated() {
    //     let data = {
    //         let mut state = ReversiState::new();
    //         state.initialize_board();
    //         MctsData::new(&state, 0, 0, None)
    //     };

    //     let tree_root = make_node(data.clone());

    //     let children = expand(tree_root.borrow())
    //         .expect("must have children")
    //         .into_iter()
    //         .collect::<Vec<_>>();

    //     assert!(
    //         !tree_root.data().is_saturated(),
    //         "Every child is saturated, but not every child has had its saturation status backpropagated, so the root should not be considered saturated."
    //     );

    //     for child in children.iter().skip(1) {
    //         backprop_saturation(child.borrow());
    //     }

    //     assert!(
    //         !tree_root.data().is_saturated(),
    //         "Every child is saturated, but not every child has had its saturation status backpropagated, so the root should not be considered saturated."
    //     );

    //     // backprop the one remaining child.
    //     backprop_saturation(children[0].borrow());

    //     assert!(
    //         tree_root.data().is_saturated(),
    //         "Now that every child has had its saturation backpropagated, the parent should be considered saturated as well."
    //     );
    // }

    // #[test]
    // fn backprop_multi_levels_works() {
    //     let data = {
    //         let mut state = ReversiState::new();
    //         state.initialize_board();
    //         MctsData::new(&state, 0, 0, None)
    //     };

    //     let tree_root = make_node(data.clone());

    //     let children_1 = expand(tree_root.borrow())
    //         .expect("must have children")
    //         .into_iter()
    //         .collect::<Vec<_>>();

    //     let child_a = &children_1[0];
    //     let child_b = &children_1[1];

    //     let grandchildren_a = expand(child_a.borrow())
    //         .unwrap()
    //         .into_iter()
    //         .collect::<Vec<_>>();

    //     let grandchildren_b = expand(child_b.borrow())
    //         .unwrap()
    //         .into_iter()
    //         .collect::<Vec<_>>();

    //     assert!(!tree_root.data().is_saturated());
    //     assert!(!child_a.borrow().data().is_saturated());
    //     assert!(!child_b.borrow().data().is_saturated());

    //     for grandchild in grandchildren_a {
    //         backprop_saturation(grandchild.borrow());
    //     }

    //     assert!(!tree_root.data().is_saturated());
    //     assert!(!child_b.borrow().data().is_saturated());

    //     assert!(child_a.borrow().data().is_saturated());

    //     for grandchild in grandchildren_b {
    //         backprop_saturation(grandchild.borrow());
    //     }

    //     assert!(!tree_root.data().is_saturated());
    //     assert!(child_a.borrow().data().is_saturated());
    //     assert!(child_b.borrow().data().is_saturated());

    //     for child in children_1.iter().skip(2) {
    //         backprop_saturation(child.borrow());
    //     }

    //     assert!(tree_root.data().is_saturated());
    //     assert!(child_a.borrow().data().is_saturated());
    //     assert!(child_b.borrow().data().is_saturated());
    // }

    // #[test]
    // fn score_node_works() {
    //     let agent = MCTSAgent::new(PlayerColor::White);
    //     let data = MctsData::new(&ReversiState::new(), 0, 0, None);

    //     let tree_root = make_node(data.clone());

    //     // all children of the same parent
    //     let child_a = tree_root.borrow().new_child(data.clone());
    //     let child_b = tree_root.borrow().new_child(data.clone());
    //     let child_c = tree_root.borrow().new_child(data.clone());
    //     let child_d = tree_root.borrow().new_child(data.clone());

    //     // "visit" each child a different amount of times
    //     agent.backprop_sim_result(child_a.borrow(), GameResult::BlackWins);
    //     agent.backprop_sim_result(child_a.borrow(), GameResult::BlackWins);
    //     agent.backprop_sim_result(child_a.borrow(), GameResult::BlackWins);

    //     agent.backprop_sim_result(child_b.borrow(), GameResult::BlackWins);
    //     agent.backprop_sim_result(child_b.borrow(), GameResult::BlackWins);

    //     agent.backprop_sim_result(child_c.borrow(), GameResult::BlackWins);

    //     assert_eq!(1.5456431, agent.score_node(child_a.borrow()));
    //     assert_eq!(1.8930185, agent.score_node(child_b.borrow()));
    //     assert_eq!(2.6771324, agent.score_node(child_c.borrow()));
    //     assert_eq!(
    //         340282350000000000000000000000000000000f32,
    //         agent.score_node(child_d.borrow())
    //     );
    // }

    // #[test]
    // fn simulate_runs_to_completion_and_terminates() {
    //     let mut initial_state = ReversiState::new();
    //     initial_state.initialize_board();
    //     let data = MctsData::new(&initial_state, 0, 0, None);

    //     let tree_root = make_node(data.clone());

    //     let _sim_result = simulate(&tree_root);
    // }
}
