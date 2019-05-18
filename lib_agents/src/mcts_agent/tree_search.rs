use lib_boardgame::game_primitives::GameState;
use std::cell::Cell;

pub trait Data<T: GameState> {
    fn state(&self) -> &T;
    fn plays(&self) -> usize;
    fn wins(&self) -> usize;
    fn action(&self) -> Option<T::Move>;
    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self;
}

/// MCTS-related data that every Node will have.
#[derive(Default, Clone)]
pub struct MctsData<T: GameState> {
    state: T,
    plays: Cell<usize>,
    wins: Cell<usize>,
    action: Option<T::Move>,

    is_expanded: Cell<bool>,

    children_count: Cell<usize>,
    children_saturated_count: Cell<usize>,
    is_saturated: Cell<bool>,
}

impl<T: GameState> MctsData<T> {
    pub fn increment_plays(&self) {
        self.plays.set(self.plays.get() + 1);
    }

    pub fn increment_wins(&self) {
        self.wins.set(self.wins.get() + 1);
    }

    /// A node is considered saturated if:
    ///     * it is a terminal node (i.e. has been expanded and still has no children), OR
    ///     * every one of its children is saturated
    /// During MCTS, we should not traverse down saturated nodes,
    /// since we have already seen every outcome.
    /// Nodes should not be marked saturated until AFTER their result
    /// has been backpropagated.
    pub fn is_saturated(&self) -> bool {
        self.is_saturated.get()
    }

    /// The owner of the tree search should call this
    /// upon expanding the node, to mark it as "expanded".
    /// This is an important because it distinguishes
    /// nodes that have been expanded but have no more children (terminal nodes)
    /// with nodes that do have possible children but have not yet been expanded (leaf nodes).
    pub fn mark_expanded(&self) {
        assert!(!self.is_expanded.get());
        self.is_expanded.set(true);
    }

    pub fn set_children_count(&self, count: usize) {
        self.children_count.set(count);
    }

    pub fn increment_saturated_children_count(&self) {
        self.children_saturated_count
            .set(self.children_saturated_count.get() + 1);

        if self.is_expanded.get()
            && self.children_saturated_count.get() == self.children_count.get()
        {
            self.is_saturated.set(true);
        }

        assert!(self.children_saturated_count.get() <= self.children_count.get());
    }
}

impl<T: GameState> Data<T> for MctsData<T> {
    fn state(&self) -> &T {
        &self.state
    }

    fn plays(&self) -> usize {
        self.plays.get()
    }

    fn wins(&self) -> usize {
        self.wins.get()
    }

    fn action(&self) -> Option<T::Move> {
        self.action
    }

    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self {
        Self {
            state: state.clone(),
            plays: Cell::new(plays),
            wins: Cell::new(wins),
            action,
            children_count: Default::default(),
            children_saturated_count: Default::default(),
            is_saturated: Cell::new(false),
            is_expanded: Cell::new(false),
        }
    }
}

#[cfg(test)]
mod tests {
    // commenting out tests until the refactoring is done
    // use super::*;
    // use lib_boardgame::game_primitives::PlayerColor;

    // #[derive(Clone)]
    // struct TestGameState;

    // impl GameState for TestGameState {
    //     type Move = usize;

    //     fn human_friendly(&self) -> String {
    //         unimplemented!()
    //     }
    //     fn initialize_board(&mut self) {
    //         unimplemented!()
    //     }
    //     fn initial_state(&self) -> String {
    //         unimplemented!()
    //     }
    //     fn legal_moves(&self, player: PlayerColor) -> String {
    //         unimplemented!()
    //     }
    //     fn apply_move(&mut self, action: Self::Move) -> String {
    //         unimplemented!()
    //     }
    //     fn current_player_turn(&self) -> String {
    //         unimplemented!()
    //     }
    //     fn player_score(&self, player: PlayerColor) {
    //         unimplemented!()
    //     }
    //     fn skip_turn(&mut self) {
    //         unimplemented!()
    //     }
    //     fn is_game_over(&self) {
    //         unimplemented!()
    //     }

    //     // note: `Move` from trait: `type Move;`
    //     // note: `human_friendly` from trait: `fn(&Self) -> std::string::String`
    //     // note: `initialize_board` from trait: `fn(&mut Self)`
    //     // note: `initial_state` from trait: `fn() -> Self`
    //     // note: `legal_moves` from trait: `fn(&Self, lib_boardgame::game_primitives::PlayerColor) -> std::vec::Vec<<Self as lib_boardgame::game_primitives::GameState>::Move>`
    //     // note: `apply_move` from trait: `fn(&mut Self, <Self as lib_boardgame::game_primitives::GameState>::Move)`
    //     // note: `current_player_turn` from trait: `fn(&Self) -> lib_boardgame::game_primitives::PlayerColor`
    //     // note: `player_score` from trait: `fn(&Self, lib_boardgame::game_primitives::PlayerColor) -> usize`
    //     // note: `skip_turn` from trait: `fn(&mut Self)`
    //     // note: `is_game_over` from trait: `fn(&Self) -> bool`
    // }

    // #[test]
    // fn is_saturated_expects_false_on_default_node() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);

    //     assert!(
    //         !data.is_saturated(),
    //         "By default, a node should not be considered saturated."
    //     );
    // }

    // #[test]
    // fn is_saturated_expects_true_for_expanded_childless_node() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);
    //     data.mark_expanded();

    //     assert!(
    //         data.is_saturated(),
    //         "An expanded node with no children should be considered saturated."
    //     );
    // }

    // #[test]
    // fn is_saturated_expects_false_for_expanded_node_with_children() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);
    //     data.mark_expanded();
    //     data.set_children_count(7);

    //     assert!(
    //         !data.is_saturated(),
    //         "An expanded node with children (that have not backprop'd their expansion status or are unexpanded) should be considered unsaturated."
    //     );
    // }

    // #[test]
    // fn is_saturated_expects_true_after_incrementing_saturation_count_fully() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);
    //     data.mark_expanded();

    //     // we mark the data as having 7 children
    //     data.set_children_count(7);

    //     (0..7).for_each(|_| data.increment_saturated_children_count());

    //     // and then "increment saturation count" 7 times

    //     assert!(
    //         data.is_saturated(),
    //         "An expanded node with a child count of 7 and a saturated-child count of 7 must therefore be considered saturated."
    //     );
    // }

    // #[test]
    // fn is_saturated_expects_false_after_incrementing_saturation_count_partially() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);
    //     data.mark_expanded();

    //     data.set_children_count(7);

    //     (0..6).for_each(|_| data.increment_saturated_children_count());

    //     assert!(
    //         data.is_saturated(),
    //         "An expanded node with a child count of 7 and a saturated-child count of 7 must therefore be considered saturated."
    //     );
    // }

    // #[test]
    // fn increment_saturated_children_count_explodes_if_over_saturated() {
    //     let data = MctsData::new(&TestGameState, 0, 0, None);
    //     data.mark_expanded();

    //     data.set_children_count(7);

    //     (0..8).for_each(|_| data.increment_saturated_children_count());

    //     assert!(
    //         data.is_saturated(),
    //         "An expanded node with a child count of 7 and a saturated-child count of 8 is impossible so we should panic."
    //     );
    // }
}
