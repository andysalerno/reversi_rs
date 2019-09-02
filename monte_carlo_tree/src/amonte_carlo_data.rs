use lib_boardgame::{GameResult, GameState};
use std::fmt;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::RwLock;
use crate::monte_carlo_data::MctsResult;

/// MCTS-related data that every Node will have.
#[derive(Default)]
pub struct AMctsData<T: GameState> {
    state: T,
    plays: AtomicUsize,
    wins: AtomicUsize,
    action: Option<T::Move>,

    is_expanded: AtomicBool,

    children_count: AtomicUsize,
    children_saturated_count: AtomicUsize,
    end_state_result: RwLock<Option<GameResult>>,
}

impl<TState> From<&AMctsData<TState>> for MctsResult<TState>
where
    TState: GameState,
{
    fn from(data: &AMctsData<TState>) -> Self {
        Self {
            plays: data.plays(),
            wins: data.wins(),
            action: data.action().expect("todo"),
            is_saturated: data.is_saturated(),
            result: None, // TODO
        }
    }
}

impl<T: GameState> AMctsData<T> {
    pub fn increment_plays(&self) {
        self.plays.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_wins(&self) {
        self.wins.fetch_add(1, Ordering::Relaxed);
    }

    /// The owner of the tree search should call this
    /// upon expanding the node, to mark it as "expanded".
    /// This is an important because it distinguishes
    /// nodes that have been expanded but have no more children (terminal nodes)
    /// with nodes that do have possible children but have not yet been expanded (leaf nodes).
    pub fn mark_expanded(&self) {
        assert!(!self.is_expanded());
        self.is_expanded.store(true, Ordering::SeqCst);
    }

    pub fn is_expanded(&self) -> bool {
        self.is_expanded.load(Ordering::SeqCst)
    }

    pub fn set_children_count(&self, count: usize) {
        self.children_count.store(count, Ordering::SeqCst);
    }

    pub fn children_count(&self) -> usize {
        self.children_count.load(Ordering::SeqCst)
    }

    pub fn increment_saturated_children_count(&self) {
        self.children_saturated_count
            .fetch_add(1, Ordering::SeqCst);

        debug_assert!(self.children_saturated_count.load(Ordering::SeqCst) <= self.children_count.load(Ordering::SeqCst));
    }

    pub fn state(&self) -> &T {
        &self.state
    }

    pub fn plays(&self) -> usize {
        self.plays.load(Ordering::SeqCst)
    }

    pub fn wins(&self) -> usize {
        self.wins.load(Ordering::SeqCst)
    }

    pub fn action(&self) -> Option<T::Move> {
        self.action
    }

    pub fn new(state: T, plays: usize, wins: usize, action: Option<T::Move>) -> Self {
        Self {
            state,
            action,

            // TODO: why can't I use the sugar `..Default::default()` for the remaining??
            plays: AtomicUsize::default(),
            wins: AtomicUsize::default(),
            is_expanded: Default::default(),
            children_count: Default::default(),
            children_saturated_count: Default::default(),
            end_state_result: Default::default(),
        }
    }

    /// A node is considered saturated if:
    ///     * it is a terminal node (i.e. has been expanded and still has no children), OR
    ///     * every one of its children is saturated
    /// During MCTS, we should not traverse down saturated nodes,
    /// since we have already seen every outcome.
    /// Nodes should not be marked saturated until AFTER their result
    /// has been backpropagated.
    pub fn is_saturated(&self) -> bool {
        let children_count = self.children_count();
        let saturated_children_count = self.children_saturated_count.load(Ordering::SeqCst);
        debug_assert!(
            saturated_children_count <= children_count,
            "Can't have more saturated children than children"
        );

        self.is_expanded() && saturated_children_count >= children_count
    }

    pub fn end_state_result(&self) -> Option<GameResult> {
        *self.end_state_result.read().unwrap()
    }

    pub fn set_end_state_result(&self, result: GameResult) {
        let mut writable = self.end_state_result.write().expect("Could not lock game result for writing.");
        *writable = Some(result);
    }
}

impl<T: GameState + fmt::Display> fmt::Display for AMctsData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;

    #[test]
    fn is_saturated_expects_false_on_default_node() {
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);

        assert!(
            !data.is_saturated(),
            "By default, a node should not be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_for_expanded_childless_node() {
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        assert!(
            data.is_saturated(),
            "An expanded node with no children should be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_false_for_expanded_node_with_children() {
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();
        data.set_children_count(7);

        assert!(
            !data.is_saturated(),
            "An expanded node with children (that have not backprop'd their expansion status or are unexpanded) should be considered unsaturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_after_incrementing_saturation_count_fully() {
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);
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
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);
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
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        data.set_children_count(7);

        (0..8).for_each(|_| data.increment_saturated_children_count());

        assert!(
            data.is_saturated(),
            "An expanded node with a child count of 7 and a saturated-child count of 8 is impossible so we should panic."
        );
    }
}
