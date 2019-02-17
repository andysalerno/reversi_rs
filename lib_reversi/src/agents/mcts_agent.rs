mod tree_search;

use lib_boardgame::game_primitives::GameResult;
use lib_boardgame::game_primitives::{GameAgent, GameState};
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
        // select
        let children = self.tree_root.children();
        let selected_child: TNode = select_child(children).unwrap();
        let _state = selected_child.data().state();

        // expand
        // let state = selected_child.unwrap().state();

        // simulate

        // backprop
        backprop(&selected_child, GameResult::BlackWins);

        legal_moves[0]
    }
}

fn select_child<TNode, TState>(nodes: TNode::ChildrenIter) -> Option<TNode>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    nodes
        .into_iter()
        .max_by(|a, b| score_node(a).partial_cmp(&score_node(b)).unwrap())
}

fn select_to_leaf<TNode, TState>(root: &TNode) -> TNode
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let cur_node = root;

    loop {
        let selected_child: Option<TNode> = select_child(root.children());

        if selected_child.is_none() {
            return cur_node.clone();
        }

        cur_node = selected_child.unwrap().clone();
    }
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
        let child = tree_root.new_child(&data);

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
    fn backprop_works_several_nodes() {
        let data = MctsData::new(ReversiState::new());

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(&data);
        let child_level_2 = child_level_1.new_child(&data);
        let child_level_3 = child_level_2.new_child(&data);
        let child_level_4 = child_level_3.new_child(&data);

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

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(&data);
        let child_level_2 = child_level_1.new_child(&data);
        let child_level_3 = child_level_2.new_child(&data);
        let child_level_4 = child_level_3.new_child(&data);

        backprop(&child_level_3, GameResult::BlackWins);

        assert_eq!(1, child_level_3.data().plays());
        assert_eq!(1, child_level_2.data().plays());
        assert_eq!(1, child_level_1.data().plays());
        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, child_level_4.data().plays());
    }

    #[test]
    fn score_child_works() {
        let data = MctsData::new(ReversiState::new());

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.new_child(&data);
        let child_b = tree_root.new_child(&data);
        let child_c = tree_root.new_child(&data);
        let child_d = tree_root.new_child(&data);

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

// impl<TState: GameState, TTree: MonteCarloTree> MCTSAgent<TState, TTree> {
//     pub fn new(game_state: &TState) -> Self {
//         Self {
//             tree: TTree::new(game_state),
//         }
//     }

//     /// Given a slice of nodes, select the node we should explore
//     /// in such a way that balances exploration and exploitation
//     /// of our state space.
//     // fn select(nodes: &[RcNode<TState>]) -> Option<&RcNode<TState>> {
//     fn select<'a>(nodes: impl Iterator<Item = &'a RcNode<TState>>) -> Option<&'a RcNode<TState>> {
//         nodes.max_by(|a, b| Self::rank_node(a).partial_cmp(&Self::rank_node(b)).unwrap())
//     }

//     fn select_to_leaf(node: &RcNode<TState>) -> Option<RcNode<TState>> {
//         let mut current_node_visiting = node;
//         let cur_ptr = None;

//         while current_node_visiting.children().borrow().len() > 0 {
//             let children_ptrs = current_node_visiting.children().borrow();
//             let children = children_ptrs.iter().map(|c| *c);

//             let selected = Self::select(children).unwrap();
//             current_node_visiting = selected;
//         }

//         Some((*current_node_visiting).clone())
//     }

// }
