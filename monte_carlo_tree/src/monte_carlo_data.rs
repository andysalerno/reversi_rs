use lib_boardgame::{GameResult, GameState};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::sync::RwLock;

#[derive(Default, Clone)]
pub struct MctsResult<TState: GameState> {
    pub result: Option<GameResult>,
    pub action: TState::Action,
    pub wins: usize,
    pub plays: usize,
    pub is_saturated: bool,
    pub terminal_count: usize,
    pub terminal_wins_count: usize,
    pub worst_wins: usize,
    pub worst_plays: usize,
    pub tree_size: usize,
    pub descendants_saturated_count: usize,
}

impl<TState> fmt::Debug for MctsResult<TState>
where
    TState: GameState,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sat_display = if self.is_saturated { " (S)" } else { "" };

        write!(
            f,
            "A: {:?} P: {:>10?} W: {:>10?} ({:.3}) TS: {:>10?} Term: {:?}/{:?} WW/WP: {}/{} Sat: {:?}{}",
            self.action,
            self.plays,
            self.wins,
            self.wins as f32 / self.plays as f32,
            self.tree_size,
            self.terminal_wins_count,
            self.terminal_count,
            self.worst_wins,
            self.worst_plays,
            self.descendants_saturated_count,
            sat_display
        )
    }
}

/// MCTS-related data that every Node will have.
#[derive(Default)]
pub struct MctsData<T>
where
    T: GameState,
{
    state: T,
    plays: AtomicUsize,
    wins: AtomicUsize,
    action: Option<T::Action>,

    is_expanded: AtomicBool,

    children_count: AtomicUsize,
    children_saturated_count: AtomicUsize,
    descendants_saturated_count: AtomicUsize,
    tree_size: AtomicUsize,
    terminal_count: AtomicUsize,
    terminal_wins_count: AtomicUsize,
    end_state_result: RwLock<Option<GameResult>>,

    /// When this subtree is fully saturated, this will hold the wins/plays
    /// of the worst-case scenario when following this path
    sat_worst_case_ratio: (AtomicUsize, AtomicUsize),
    sim_lock: Mutex<()>,
}

impl<T> fmt::Debug for MctsData<T>
where
    T: GameState,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Action: {:?} Plays: {:?} Wins: {:?} ({}) Treesize: {:?}",
            self.action, self.plays, self.wins, 0.00, self.tree_size
        )
    }
}

impl<TState> Clone for MctsData<TState>
where
    TState: GameState,
{
    fn clone(&self) -> Self {
        let end_state_result = self
            .end_state_result
            .read()
            .expect("Couldn't acquire read lock on end state result.");
        let end_state_result = RwLock::new(*end_state_result);

        let plays = clone_atomic_usize(&self.plays);
        let wins = clone_atomic_usize(&self.wins);
        let children_count = clone_atomic_usize(&self.children_count);
        let children_saturated_count = clone_atomic_usize(&self.children_saturated_count);
        let descendants_saturated_count = clone_atomic_usize(&self.descendants_saturated_count);
        let tree_size = clone_atomic_usize(&self.tree_size);
        let terminal_count = clone_atomic_usize(&self.terminal_count);
        let terminal_wins_count = clone_atomic_usize(&self.terminal_wins_count);
        let sat_worst_case_ratio = (
            clone_atomic_usize(&self.sat_worst_case_ratio.0),
            clone_atomic_usize(&self.sat_worst_case_ratio.1),
        );

        Self {
            state: self.state.clone(),
            action: self.action,
            end_state_result,
            plays,
            wins,
            children_count,
            children_saturated_count,
            is_expanded: AtomicBool::new(self.is_expanded()),
            tree_size,
            terminal_count,
            sat_worst_case_ratio,
            descendants_saturated_count,
            terminal_wins_count,
            sim_lock: Mutex::new(()),
        }
    }
}

fn clone_atomic_usize(atom: &AtomicUsize) -> AtomicUsize {
    let raw = atom.load(Ordering::SeqCst);
    AtomicUsize::new(raw)
}

impl<TState> From<&MctsData<TState>> for MctsResult<TState>
where
    TState: GameState,
{
    fn from(data: &MctsData<TState>) -> Self {
        let (wwins, wplays) = data.worst_case_wins_plays();

        Self {
            plays: data.plays(),
            wins: data.wins(),
            action: data
                .action()
                .expect("can't convert to MctsResult without an action"),
            is_saturated: data.is_saturated(),
            result: None, // TODO,
            tree_size: data.tree_size(),
            terminal_count: data.terminal_count(),
            terminal_wins_count: data.terminal_wins_count(),
            descendants_saturated_count: data.descendants_saturated_count(),
            worst_wins: wwins,
            worst_plays: wplays,
        }
    }
}

impl<T> MctsData<T>
where
    T: GameState,
{
    pub fn new(state: T, plays: usize, wins: usize, action: Option<T::Action>) -> Self {
        Self {
            state,
            action,

            // TODO: why can't I use the sugar `..Default::default()` for the remaining??
            plays: AtomicUsize::new(plays),
            wins: AtomicUsize::new(wins),
            is_expanded: AtomicBool::new(false),
            children_count: Default::default(),
            children_saturated_count: Default::default(),
            descendants_saturated_count: Default::default(),
            end_state_result: Default::default(),
            tree_size: Default::default(),
            terminal_count: Default::default(),
            terminal_wins_count: Default::default(),
            sat_worst_case_ratio: (Default::default(), Default::default()),
            sim_lock: Mutex::new(()),
        }
    }

    // "Read" functions

    pub fn state(&self) -> &T {
        &self.state
    }

    pub fn get_lock(&self) -> &std::sync::Mutex<()> {
        &self.sim_lock
    }

    pub fn plays(&self) -> usize {
        self.plays.load(Ordering::SeqCst)
    }

    pub fn wins(&self) -> usize {
        self.wins.load(Ordering::SeqCst)
    }

    pub fn action(&self) -> Option<T::Action> {
        self.action
    }

    pub fn tree_size(&self) -> usize {
        self.tree_size.load(Ordering::SeqCst)
    }

    pub fn is_expanded(&self) -> bool {
        self.is_expanded.load(Ordering::SeqCst)
    }

    pub fn children_count(&self) -> usize {
        self.children_count.load(Ordering::SeqCst)
    }

    pub fn descendants_saturated_count(&self) -> usize {
        self.descendants_saturated_count.load(Ordering::SeqCst)
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

    pub fn terminal_count(&self) -> usize {
        self.terminal_count.load(Ordering::SeqCst)
    }

    pub fn terminal_wins_count(&self) -> usize {
        self.terminal_wins_count.load(Ordering::SeqCst)
    }

    pub fn end_state_result(&self) -> Option<GameResult> {
        *self.end_state_result.read().unwrap()
    }

    pub fn worst_case_wins_plays(&self) -> (usize, usize) {
        (
            self.sat_worst_case_ratio.0.load(Ordering::SeqCst),
            self.sat_worst_case_ratio.1.load(Ordering::SeqCst),
        )
    }

    // "Write" functions

    /// The owner of the tree search should call this
    /// upon expanding the node, to mark it as "expanded".
    /// This is an important because it distinguishes
    /// nodes that have been expanded but have no more children (terminal nodes)
    /// with nodes that do have possible children but have not yet been expanded (leaf nodes).
    pub fn mark_expanded(&self) {
        self.is_expanded.store(true, Ordering::SeqCst);
    }

    pub fn set_children_count(&self, count: usize) {
        self.children_count.store(count, Ordering::SeqCst);
    }

    pub fn increment_saturated_children_count(&self) {
        let children_count = self.children_count.load(Ordering::SeqCst);
        let new_sat_count = 1 + self.children_saturated_count.fetch_add(1, Ordering::SeqCst);

        assert!(
            new_sat_count <= children_count,
            "can never increment saturated children beyond the count of all children. node action: {:?} new_sat_count: {}, children_count: {}",
            self.action(), new_sat_count, children_count
        );
    }

    pub fn increment_descendants_saturated_count(&self, by_count: usize) {
        self.descendants_saturated_count
            .fetch_add(by_count, Ordering::SeqCst);
    }

    pub fn increment_terminal_count(&self, is_win: bool) {
        self.terminal_count.fetch_add(1, Ordering::SeqCst);

        if is_win {
            self.terminal_wins_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn increment_tree_size(&self, count: usize) {
        self.tree_size.fetch_add(count, Ordering::SeqCst);
    }

    pub fn increment_plays(&self) {
        self.plays.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_wins(&self) {
        self.wins.fetch_add(1, Ordering::Relaxed);
    }

    /// Updates the current worst case wins/plays ratio,
    /// if the given wins/plays ratio is better than the current worst case.
    pub fn update_worst_case(&self, wins: usize, plays: usize) {
        if plays == 0 {
            return;
        }

        // Might need to lock this critical chunk
        let cur_wins = self.sat_worst_case_ratio.0.load(Ordering::SeqCst);
        let cur_plays = self.sat_worst_case_ratio.1.load(Ordering::SeqCst);

        if cur_plays == 0 || ((wins as f32 / plays as f32) < (cur_wins as f32 / cur_plays as f32)) {
            self.sat_worst_case_ratio.0.store(wins, Ordering::SeqCst);
            self.sat_worst_case_ratio.1.store(plays, Ordering::SeqCst);
        }
    }

    pub fn set_end_state_result(&self, result: GameResult) {
        let mut writable = self
            .end_state_result
            .write()
            .expect("Could not lock game result for writing.");
        *writable = Some(result);
    }
}

impl<T> fmt::Display for MctsData<T>
where
    T: GameState + fmt::Display,
{
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
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);

        assert!(
            !data.is_saturated(),
            "By default, a node should not be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_for_expanded_childless_node() {
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        assert!(
            data.is_saturated(),
            "An expanded node with no children should be considered saturated."
        );
    }

    #[test]
    fn is_saturated_expects_false_for_expanded_node_with_children() {
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();
        data.set_children_count(7);

        assert!(
            !data.is_saturated(),
            "An expanded node with children (that have not backprop'd their expansion status or are unexpanded) should be considered unsaturated."
        );
    }

    #[test]
    fn is_saturated_expects_true_after_incrementing_saturation_count_fully() {
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);
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
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);
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
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);
        data.mark_expanded();

        data.set_children_count(7);

        (0..8).for_each(|_| data.increment_saturated_children_count());

        assert!(
            data.is_saturated(),
            "An expanded node with a child count of 7 and a saturated-child count of 8 is impossible so we should panic."
        );
    }
}
