use lib_boardgame::{GameResult, GameState};
use std::cell::Cell;

pub trait Data<T: GameState> {
    fn state(&self) -> &T;
    fn plays(&self) -> usize;
    fn wins(&self) -> usize;
    fn action(&self) -> Option<T::Move>;
    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self;
    fn end_state_result(&self) -> Option<GameResult>;
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
    end_state_result: Cell<Option<GameResult>>,
}

#[derive(Default)]
pub struct MctsResult<TState: GameState> {
    pub result: Option<GameResult>,
    pub action: TState::Move,
    pub wins: usize,
    pub plays: usize,
}

impl<TData, TState> From<&TData> for MctsResult<TState>
where 
TData: Data<TState>,
TState: GameState,
{
    fn from(data: &TData) -> Self {
        Self {
            plays: data.plays(),
            wins: data.wins(),
            result: data.end_state_result(), // TODO
            action: data.action().expect("todo"),
        }
    }
}

impl<T: GameState> MctsData<T> {
    pub fn increment_plays(&self) {
        self.plays.set(self.plays.get() + 1);
    }

    pub fn increment_wins(&self) {
        self.wins.set(self.wins.get() + 1);
    }

    pub fn end_state_result(&self) -> Option<GameResult> {
        self.end_state_result.get()
    }

    pub fn set_end_state_result(&self, result: GameResult) {
        self.end_state_result.set(Some(result));
    }

    /// A node is considered saturated if:
    ///     * it is a terminal node (i.e. has been expanded and still has no children), OR
    ///     * every one of its children is saturated
    /// During MCTS, we should not traverse down saturated nodes,
    /// since we have already seen every outcome.
    /// Nodes should not be marked saturated until AFTER their result
    /// has been backpropagated.
    pub fn is_saturated(&self) -> bool {
        self.is_expanded.get() && self.children_saturated_count.get() >= self.children_count.get()
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

    fn end_state_result(&self) -> Option<GameResult> {
        self.end_state_result.get()
    }

    fn new(state: &T, plays: usize, wins: usize, action: Option<T::Move>) -> Self {
        Self {
            state: state.clone(),
            plays: Cell::new(plays),
            wins: Cell::new(wins),
            action,
            children_count: Default::default(),
            children_saturated_count: Default::default(),
            is_expanded: Cell::new(false),
            end_state_result: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

    #[test]
    fn is_saturated_expects_false_on_default_node() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);

        assert!(
            !data.is_saturated(),
            "By default, a node should not be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_for_expanded_childless_node() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        assert!(
            data.is_saturated(),
            "An expanded node with no children should be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_false_for_expanded_node_with_children() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();
        data.set_children_count(7);

        assert!(
            !data.is_saturated(),
            "An expanded node with children (that have not backprop'd their expansion status or are unexpanded) should be considered unsaturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_after_incrementing_saturation_count_fully() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        // we mark the data as having 7 children
        data.set_children_count(7);

        // and then "increment saturation count" 7 times
        (0..7).for_each(|_| data.increment_saturated_children_count());

        assert!(
            data.is_saturated(),
            "An expanded node with a child count of 7 and a saturated-child count of 7 must therefore be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_false_after_incrementing_saturation_count_partially() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        data.set_children_count(7);

        (0..6).for_each(|_| data.increment_saturated_children_count());

        assert!(
            !data.is_saturated(),
            "A node with 7 children, but only 6 saturated children, should not be considered saturated."
        );
    }

    #[test]
    #[should_panic]
    fn increment_saturated_children_count_explodes_if_over_saturated() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        data.set_children_count(7);

        (0..8).for_each(|_| data.increment_saturated_children_count());

        assert!(
            data.is_saturated(),
            "An expanded node with a child count of 7 and a saturated-child count of 8 is impossible so we should panic."
        );
    }
}
