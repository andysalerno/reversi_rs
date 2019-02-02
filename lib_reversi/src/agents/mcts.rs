use lib_boardgame::game_primitives::{GameAgent, GameState};
use monte_carlo_tree::rc_tree::{RcNode, RcTree};

pub struct MCTSAgent<TState: GameState> {
    tree: RcTree<TState>,
}

impl<TState: GameState> GameAgent<TState> for MCTSAgent<TState> {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        unimplemented!();

        // select
        let children_ptrs = self.tree.root().children().borrow();
        let children = {
            let c = Vec::new();
            for child in children_ptrs.iter() {
                let r = **child;
                c.push(r);
            }

            c
        };
        let selected_child = Self::select(children.as_slice().iter());

        // expand
        let state = selected_child.unwrap().state();

        // simulate

        // backprop
    }
}

impl<TState: GameState> MCTSAgent<TState> {
    pub fn new(game_state: &TState) -> Self {
        Self {
            tree: RcTree::<TState>::new(game_state.clone()),
        }
    }

    /// Given a slice of nodes, select the node we should explore
    /// in such a way that balances exploration and exploitation
    /// of our state space.
    // fn select(nodes: &[RcNode<TState>]) -> Option<&RcNode<TState>> {
    fn select<'a>(nodes: impl Iterator<Item = &'a RcNode<TState>>) -> Option<&'a RcNode<TState>> {
        nodes.max_by(|a, b| Self::rank_node(a).partial_cmp(&Self::rank_node(b)).unwrap())
    }

    fn select_to_leaf(node: &RcNode<TState>) -> Option<RcNode<TState>> {
        let mut current_node_visiting = node;
        let cur_ptr = None;

        while current_node_visiting.children().borrow().len() > 0 {
            let children_ptrs = current_node_visiting.children().borrow();
            let children = children_ptrs.iter().map(|c| *c);

            let selected = Self::select(children).unwrap();
            current_node_visiting = selected;
        }

        Some((*current_node_visiting).clone())
    }

    /// Given a node, score it in such a way that encourages
    /// both exploration and exploitation of the state space.
    fn rank_node(node: &RcNode<TState>) -> f32 {
        let plays = node.plays() as f32;

        if plays == 0f32 {
            return std::f32::MAX;
        }

        let wins = node.wins() as f32;
        let parent_plays = node.parent().upgrade().map_or(0, |p| p.plays()) as f32;
        let bias = 2 as f32;

        (wins / plays) + (bias * f32::sqrt(f32::ln(parent_plays) / plays))
    }
}
