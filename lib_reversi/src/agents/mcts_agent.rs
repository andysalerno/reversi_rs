mod tree_search;

use lib_boardgame::game_primitives::GameResult;
use lib_boardgame::game_primitives::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::Node;
use std::borrow::Borrow;
use tree_search::{Data, MctsData};

pub struct MCTSAgent<TTree: Node> {
    tree_root: TTree,
}

impl<TNode, TState> GameAgent<TState> for MCTSAgent<TNode>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        for _ in 0..1000 {
            // select
            let child_borrowable = select_to_leaf::<TNode, TState>(&self.tree_root);
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
            backprop(selected, sim_result);
        }

        legal_moves[0]
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
    let legal_actions = state.legal_moves(PlayerColor::Black);

    for action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = MctsData::new(resulting_state);
        let _child_node = node.new_child(data);
    }

    node.children()
}

fn simulate<TNode, TState>(node: &TNode) -> GameResult
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    GameResult::WhiteWins
}

fn backprop<TNode, TState>(node: &TNode, _result: GameResult)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    node.data().increment_plays();
    node.data().increment_wins();

    let mut parent_node = node.parent();

    while let Some(p) = parent_node {
        let parent = p.borrow();
        parent.data().increment_plays();
        parent.data().increment_wins();
        parent_node = parent.parent();
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
        let data = MctsData::new(ReversiState::new());
        let tree_root = make_node(data.clone());
        let child = tree_root.new_child(data.clone());

        assert_eq!(1, tree_root.children().into_iter().count());
        assert!(child.parent().is_some());
        assert!(tree_root.parent().is_none());
    }

    #[test]
    fn backprop_works_one_node() {
        let data = MctsData::new(ReversiState::new());
        let tree_root = make_node(data.clone());

        backprop(&tree_root, GameResult::BlackWins);

        assert_eq!(1, tree_root.data().plays());
    }

    #[test]
    fn expand_works() {
        let mut initial_state = ReversiState::new();

        // default Reversi initial configuration
        initial_state.initialize_board();

        let data = MctsData::new(initial_state);
        let tree_root = RcNode::new_root(data);

        let expanded_children = expand::<RcNode<MctsData<ReversiState>>, ReversiState>(&tree_root)
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(4, expanded_children.len());
    }

    #[test]
    fn backprop_works_several_nodes() {
        let data = MctsData::new(ReversiState::new());

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
        let data = MctsData::new(ReversiState::new());

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
        let data = MctsData::new(ReversiState::new());

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
        let data = MctsData::new(ReversiState::new());

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
}
