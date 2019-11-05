/// When we used "saturation", we ran many times more
/// unique simulations, and explored without duplicating any work.
/// But it gave larger subtrees preference over smaller ones which
/// might have been better.
/// Without "saturation", we keep re-playing paths we've already been done.
/// What we need is for every parent to hold not only "saturated_children / all_children",
/// but those same numbers for all descendents of its entire subtree.
/// Then we can know, way up at the root, how well explored the choice really is.

/// Better idea:
/// backprop "worst case" scenarios from the bottom when saturated
/// I.e. every child node backprops its worst case scenario
use std::borrow::Borrow;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crossbeam::thread;

use crate::util;
use lib_boardgame::{GameResult, GameState, PlayerColor};
use lib_printer::{out, out_impl};
use monte_carlo_tree::{monte_carlo_data::MctsData, monte_carlo_data::MctsResult, tree::Node};

mod configs {
    pub(super) const SIM_TIME_MS: u64 = 5_000;
    pub(super) const EXTRA_TIME_MS: u64 = 0_000;

    pub(super) const BLACK_FILTER_SAT: bool = true;
    pub(super) const WHITE_FILTER_SAT: bool = true;

    pub(super) const BLACK_THREAD_COUNT: usize = 8;
    pub(super) const WHITE_THREAD_COUNT: usize = 8;
}

fn expand<TNode, TState>(node: &TNode) -> Result<(), &str>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    // Acquire the write lock on the children
    let children_write_lock = node.children_write_lock();

    // Critical lock scope of this function:
    {
        if node.data().is_expanded() {
            return Err("We acquired the lock, but the previous holder already expanded.");
        }

        node.data().mark_expanded();

        let state = node.data().state();
        if state.is_game_over() {
            // if the game is over, we have nothing to expand
            node.data().set_children_count(0);
            return Ok(());
        }

        // TODO: There's no reason for legal_moves() to need this argument
        // since the state already knows the player's turn.
        let player_turn = state.current_player_turn();
        let legal_actions = state.legal_moves(player_turn);

        // Now that we've expanded this node, update it to
        // inform it how many children it has.
        node.data().set_children_count(legal_actions.len());
        backprop_increment_tree_size(node, legal_actions.len());

        let new_children = legal_actions
            .iter()
            .map(|&a| node.new_child(MctsData::new(state.next_state(a), 0, 0, Some(a))))
            .collect::<Vec<_>>();

        children_write_lock.write(new_children);
    }

    drop(children_write_lock);

    Ok(())
}

/// Increment this node's count of saturated children.
/// If doing so results in this node itself becoming saturated,
/// follow the same operation for its parent.
fn backprop_saturation<TNode, TState>(leaf: &TNode)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    debug_assert!(
        leaf.data().is_saturated(),
        "Only a leaf considered saturated can have its saturated status backpropagated."
    );

    let mut count = 1;
    let mut continuous_saturation = true;
    let (mut wins, mut plays) = (leaf.data().wins(), leaf.data().plays());
    leaf.data().update_worst_case(wins, plays);

    let mut handle = leaf.parent();

    while let Some(p) = handle {
        let node = p.borrow();
        let data = node.data();
        let _lock = data.get_lock().lock();

        let was_saturated_before = data.is_saturated();

        if continuous_saturation {
            data.increment_saturated_children_count();
            data.update_worst_case(wins, plays);
        }

        data.increment_descendants_saturated_count(count);

        let was_saturated_after = data.is_saturated();
        wins = data.wins();
        plays = data.plays();

        if !was_saturated_before && was_saturated_after {
            count += 1;
        }

        if !data.is_saturated() {
            // Don't back-prop any further
            // if we've reached a non-saturated node.
            continuous_saturation = false;
            // return;
        }

        handle = node.parent();
    }
}

// TODO: this same work can be done while we are already doing increment_saturation_count()
fn backprop_terminal_count<TNode, TState>(leaf: &TNode, is_win: bool)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    debug_assert!(
        leaf.data().is_saturated(),
        "Only a leaf considered saturated can have its saturated status backpropagated."
    );

    debug_assert_eq!(
        leaf.data().plays(),
        1,
        "A terminal leaf we are backpropping must have been played exactly once."
    );

    let mut handle = Some(leaf.get_handle());

    while let Some(p) = handle {
        let node = p.borrow();
        let data = node.data();

        data.increment_terminal_count(is_win);

        handle = node.parent();
    }
}

fn backprop_sim_result<TNode, TState>(node: &TNode, is_win: bool)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut handle = Some(node.get_handle());

    while let Some(p) = handle {
        let parent = p.borrow();
        let data = parent.data();
        data.increment_plays();

        if is_win {
            data.increment_wins();
        }

        handle = parent.parent();
    }
}

fn backprop_increment_tree_size<TNode, TState>(node: &TNode, by_count: usize)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut handle = Some(node.get_handle());

    while let Some(p) = handle {
        let parent = p.borrow();
        let data = parent.data();

        data.increment_tree_size(by_count);

        handle = parent.parent();
    }
}

fn simulate<TNode, TState, R>(node: &TNode, rng: &mut R) -> GameResult
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    R: rand::Rng,
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
        let random_action = util::random_choice(&legal_moves, rng);

        state.apply_move(random_action);
    }
}

/// Selects using max UCB, but on opponent's turn inverts the score.
/// If the given node has no unsaturated children,
/// returns a handle back to the given node.
fn select_to_leaf<TNode, TState>(root: &TNode, player_color: PlayerColor) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    while let Some(c) = select_child_for_traversal::<TNode, TState>(cur_node.borrow(), player_color)
    {
        cur_node = c;
    }

    cur_node
}

/// Returns a handle to the child with the greatest selection score,
/// or None if there are no children OR all children have been saturated.
fn select_child_for_traversal<TNode, TState>(
    root: &TNode,
    player_color: PlayerColor,
) -> Option<TNode::Handle>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let parent_data = root.data();
    let parent_is_player_color = parent_data.state().current_player_turn() == player_color;
    let parent_plays = parent_data.plays();
    let parent_plays = usize::max(1, parent_plays);

    let child_nodes = root.children_read();

    let filter_sat = match player_color {
        PlayerColor::Black => configs::BLACK_FILTER_SAT,
        PlayerColor::White => configs::WHITE_FILTER_SAT,
    };

    (*child_nodes)
        .iter()
        .filter(|&n| !filter_sat || !n.borrow().data().is_saturated())
        .max_by(|&a, &b| {
            let a_score =
                score_node_for_traversal(a.borrow(), parent_plays, parent_is_player_color);
            let b_score =
                score_node_for_traversal(b.borrow(), parent_plays, parent_is_player_color);

            a_score.partial_cmp(&b_score).unwrap()
        })
        .and_then(|n| Some(n.clone()))
}

fn score_node_for_traversal<TNode, TState>(
    node: &TNode,
    parent_plays: usize,
    parent_is_player_color: bool,
) -> f32
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let data = node.data();
    let plays = data.plays() as f32;

    if plays == 0f32 {
        return std::f32::MAX;
    }

    let (wwins, wplays) = data.worst_case_wins_plays();
    if wplays > 0 && wwins == 0 {
        // the worst case is a loss. don't take it.
        return std::f32::MIN;
    }

    // Experiment
    let wins = if parent_is_player_color {
        data.wins() as f32
    } else {
        let wins = data.wins();
        let plays = data.plays();

        debug_assert!(plays >= wins);

        (plays - wins) as f32
    };

    let parent_plays = parent_plays as f32;

    let node_mean_val = wins / plays;

    let explore_bias = 1.60;

    let score = node_mean_val + (explore_bias * f32::sqrt(f32::ln(parent_plays) / plays));

    if score.is_nan() {
        panic!(
            "plays: {}\nwins: {}\nparent_plays: {}\nparent_is_player_color: {}",
            plays, wins, parent_plays, parent_is_player_color
        );
    }

    score
}

pub fn mcts_result<TNode, TState>(
    root_handle: TNode::Handle,
    player_color: PlayerColor,
) -> Vec<MctsResult<TState>>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let root = root_handle.borrow();
    out!(
        "Beginning mcts on node with wins/plays: {}/{}",
        root.data().wins(),
        root.data().plays()
    );

    mcts(root, player_color);

    let mut state_children = root.children_read().iter().cloned().collect::<Vec<_>>();

    if root.data().is_saturated() {
        state_children
            .sort_by_key(|c| (c.borrow().data().wins() * 10000) / c.borrow().data().plays());
    } else {
        // state_children.sort_by_key(|c| c.borrow().data().plays());
        // TODO experimenting here
        state_children
            .sort_by_key(|c| (c.borrow().data().wins() * 10000) / c.borrow().data().plays());
    };

    // Regardless of any other metric, actions that win the game are always preferred.
    state_children.sort_by_key(|c| {
        if let Some(result) = c.borrow().data().end_state_result() {
            result.is_win_for_player(player_color)
        } else {
            false
        }
    });

    state_children
        .into_iter()
        .map(|c| c.borrow().data().into())
        .collect()
}

fn mcts<TNode, TState>(root: &TNode, player_color: PlayerColor)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let thread_count = match player_color {
        PlayerColor::Black => configs::BLACK_THREAD_COUNT,
        PlayerColor::White => configs::WHITE_THREAD_COUNT,
    };

    if thread_count == 1 {
        mcts_loop(root, player_color);
    } else {
        thread::scope(|s| {
            for _ in 0..thread_count {
                s.spawn(move |_| {
                    mcts_loop(root, player_color);
                });
            }
        })
        .unwrap();
    }
}

fn mcts_loop<TNode, TState>(root: &TNode, player_color: PlayerColor)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let now = Instant::now();
    let exec_duration = Duration::from_millis(configs::SIM_TIME_MS);
    let extra_time = Duration::from_millis(configs::EXTRA_TIME_MS);

    let mut rng = util::get_rng();

    loop {
        if now.elapsed() >= exec_duration {
            let data = root.data();

            if (data.wins() * 1000) / data.plays() > 800
                || now.elapsed() >= exec_duration + extra_time
            {
                break;
            }
        }

        if root.data().is_saturated() {
            break;
        }

        let leaf = select_to_leaf(root, player_color);
        let leaf = leaf.borrow();

        if expand(leaf).is_err() {
            // another thread beat us to expanding,
            // so just continue with a new leaf selection
            continue;
        }

        let expanded_children = leaf.children_read();

        // Here's the race condition bug:
        // ThreadA expanded, then enters the "true" part of this if block,
        // and performs the behavior on the expanded node's child.
        // ThreadB selected to leaf, and selected the same node's child.
        // This child node happens to be a terminal.
        // Now we are executing the "true" and "false" blocks
        // simultaneously for the same node.
        if !expanded_children.is_empty() {
            let sim_node = util::random_pick(expanded_children.as_slice(), &mut rng)
                .expect("Must have had at least one expanded child.");
            let sim_node = sim_node.borrow();

            run_locked_if(
                sim_node.data().get_lock(),
                || sim_node.data().plays() == 0,
                || {
                    let sim_result = simulate(sim_node, &mut rng);

                    let is_win = sim_result.is_win_for_player(player_color);
                    backprop_sim_result(sim_node, is_win);
                },
            );
        } else {
            // This whole section needs its own double-checked lock.

            // We expanded the node, but it had no children,
            // so this node must be a terminating node.
            let sim_result = simulate(leaf, &mut rng);

            // plays could be 0 or 1
            // 0 if the parent node was expanded, and sim'd on a different child
            // 1 if the parent node was expanded, and sim'd on this child
            // if this is our first time selecting this node...
            let is_win = sim_result.is_win_for_player(player_color);

            run_locked_if(
                leaf.data().get_lock(),
                || leaf.data().plays() == 0,
                || {
                    backprop_sim_result(leaf, is_win);
                },
            );

            run_locked_if(
                leaf.data().get_lock(),
                || leaf.data().end_state_result().is_none(),
                || {
                    // TODO: data race possible here? I check if it's none,
                    // then I set. But if two saw none, both set (only one actually sets),
                    // but then both still do the backprop saturation logic. Need lock here?
                    // bit of a hack, this is just to know we've never done this before
                    // Update the terminating node so it knows its own end game result.
                    leaf.data().set_end_state_result(sim_result);

                    // TODO: these two guys can be combined
                    backprop_saturation(leaf);
                    backprop_terminal_count(leaf, is_win);
                },
            );
        }
    }
}

/// If the condition is true, acquires the lock, then confirms the condition is still true
/// (in case of a race condition), and if still true, executes the action.
fn run_locked_if<F1, F2, T>(lock: &Mutex<T>, condition: F1, action: F2)
where
    F1: Fn() -> bool,
    F2: FnOnce(),
{
    if condition() {
        let lock_guard = lock.lock();

        if condition() {
            action();
        }

        drop(lock_guard);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use monte_carlo_tree::tree::Node;

    use lib_tic_tac_toe::tic_tac_toe_gamestate::{TicTacToeAction, TicTacToeState};

    use std::str::FromStr;

    use monte_carlo_tree::arc_tree::ArcNode;

    fn make_test_state() -> impl GameState {
        TicTacToeState::initial_state()
    }

    fn make_node<G>(data: MctsData<G>) -> impl Node<Data = MctsData<G>>
    where
        G: GameState + Sync + Send,
        G::Move: Sync + Send,
    {
        ArcNode::new_root(data)
    }

    fn add_children_to_parent<TNode, TState>(parent: &TNode, children: Vec<TNode::Handle>)
    where
        TNode: Node<Data = MctsData<TState>>,
        TState: GameState,
    {
        let parent_write_lock = parent.children_write_lock();
        parent_write_lock.write(children);
    }

    fn make_test_data() -> MctsData<TicTacToeState> {
        MctsData::new(TicTacToeState::initial_state(), 0, 0, None)
    }

    #[test]
    fn new_child_expects_add_child_to_parent() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        let root_children_lock = tree_root.children_write_lock();
        let child = tree_root.new_child(data.clone());
        let children = vec![child.borrow().get_handle()];
        root_children_lock.write(children);

        assert_eq!(1, tree_root.children_read().len());
        assert!(child.borrow().parent().is_some());
        assert!(tree_root.parent().is_none());
    }

    #[test]
    fn backprop_sim_results_when_black_wins_expects_update_plays_wins() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());
        let is_win = true;

        backprop_sim_result(&tree_root, is_win);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(1, tree_root.data().wins());
    }

    #[test]
    fn backprop_sim_results_when_black_wins_expects_update_white_plays_not_wins() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());
        let is_win = false;

        backprop_sim_result(&tree_root, is_win);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, tree_root.data().wins());
    }

    #[test]
    fn backprop_sim_results_expects_updates_to_root() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.borrow().new_child(data.clone());
        let child_level_3 = child_level_2.borrow().new_child(data.clone());
        let child_level_4 = child_level_3.borrow().new_child(data.clone());

        let is_win = true;
        backprop_sim_result(child_level_3.borrow(), is_win);

        assert_eq!(1, child_level_3.borrow().data().plays());
        assert_eq!(1, child_level_2.borrow().data().plays());
        assert_eq!(1, child_level_1.borrow().data().plays());
        assert_eq!(1, tree_root.data().plays());

        assert_eq!(1, child_level_3.borrow().data().wins());
        assert_eq!(1, child_level_2.borrow().data().wins());
        assert_eq!(1, child_level_1.borrow().data().wins());
        assert_eq!(1, tree_root.data().wins());

        assert_eq!(0, child_level_4.borrow().data().plays());
    }

    #[test]
    fn expand_expects_creates_children() {
        let tree_root = ArcNode::new_root(make_test_data());

        expand(&tree_root).unwrap();
        let children = tree_root.children_read();
        let children = children.iter().cloned();

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, children.len());
    }

    #[test]
    fn expand_expects_adds_children_to_parent() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.children_read().len());

        expand(&tree_root).unwrap();

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, tree_root.children_read().len());
    }

    #[test]
    fn expand_expects_marks_node_expanded() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert!(!tree_root.data().is_expanded());

        expand(&tree_root).unwrap();

        assert!(tree_root.data().is_expanded());
    }

    #[test]
    fn expand_expects_updates_children_count() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.data().children_count());

        expand(&tree_root).unwrap();

        assert_eq!(9, tree_root.data().children_count());
    }

    #[test]
    fn select_child_max_score_expects_picks_less_explored_node() {
        let data = MctsData::new(TicTacToeState::new(), 0, 0, None);

        let tree_root = ArcNode::new_root(data.clone());

        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_1: &ArcNode<_> = child_level_1.borrow();
        add_children_to_parent(&tree_root, vec![child_level_1.get_handle()]);

        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_2: &ArcNode<_> = child_level_2.borrow();
        add_children_to_parent(child_level_1, vec![child_level_2.get_handle()]);

        let child_level_3_handle = child_level_2.new_child(data.clone());
        let child_level_3: &ArcNode<_> = child_level_3_handle.borrow();
        add_children_to_parent(child_level_2, vec![child_level_3.get_handle()]);

        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4: &ArcNode<_> = child_level_4.borrow();

        let child_level_4b = child_level_3.new_child(data.clone());
        let child_level_4b: &ArcNode<_> = child_level_4b.borrow();
        add_children_to_parent(
            child_level_3,
            vec![child_level_4.get_handle(), child_level_4b.get_handle()],
        );

        // TODO: remove when we set this in the write() on the lock
        child_level_1.data().set_children_count(1);
        child_level_2.data().set_children_count(1);
        child_level_3.data().set_children_count(2);
        child_level_4.data().set_children_count(1);
        child_level_4b.data().set_children_count(1);

        let is_win = true;
        backprop_sim_result(child_level_3, is_win);
        backprop_sim_result(child_level_4, is_win);
        backprop_sim_result(child_level_4, is_win);
        backprop_sim_result(child_level_4, is_win);
        backprop_sim_result(child_level_4b, is_win);

        assert!(!child_level_3.data().is_saturated());

        let selected = select_child_for_traversal::<ArcNode<_>, TicTacToeState>(
            child_level_3_handle.borrow(),
            PlayerColor::Black,
        )
        .expect("the child should have been selected.");

        let selected: &ArcNode<_> = selected.borrow();

        assert_eq!(1, selected.data().plays());
    }

    #[test]
    fn select_to_leaf_expects_selects_less_explored_path() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        add_children_to_parent(&tree_root, vec![child_level_1.clone()]);

        let child_level_2 = child_level_1.borrow().new_child(data.clone());
        add_children_to_parent(child_level_1.borrow(), vec![child_level_2.clone()]);

        let child_level_3 = child_level_2.borrow().new_child(data.clone());
        add_children_to_parent(child_level_2.borrow(), vec![child_level_3.clone()]);

        let child_level_4 = child_level_3.borrow().new_child(data.clone());

        let child_level_4b = child_level_3.borrow().new_child(data.clone());
        add_children_to_parent(
            child_level_3.borrow(),
            vec![child_level_4.clone(), child_level_4b.clone()],
        );

        tree_root.data().set_children_count(1);
        child_level_1.borrow().data().set_children_count(1);
        child_level_2.borrow().data().set_children_count(1);
        child_level_3.borrow().data().set_children_count(2);
        child_level_4.borrow().data().set_children_count(2);
        child_level_4b.borrow().data().set_children_count(2);

        let is_win = true;
        backprop_sim_result(child_level_3.borrow(), is_win);
        backprop_sim_result(child_level_4.borrow(), is_win);
        backprop_sim_result(child_level_4.borrow(), is_win);
        backprop_sim_result(child_level_4.borrow(), is_win);
        backprop_sim_result(child_level_4.borrow(), is_win);
        backprop_sim_result(child_level_4b.borrow(), is_win);
        backprop_sim_result(child_level_4b.borrow(), is_win);

        let leaf = select_to_leaf(&tree_root, PlayerColor::Black);

        let leaf = leaf.borrow();

        assert_eq!(2, leaf.data().plays());
    }

    #[test]
    fn select_to_leaf_expects_when_already_leaf_returns_self() {
        let data = MctsData::new(TicTacToeState::initial_state(), 10, 10, None);

        let tree_root = make_node(data.clone());

        let leaf = select_to_leaf(&tree_root, PlayerColor::Black);
        let leaf = leaf.borrow();

        assert_eq!(10, leaf.data().plays());
        assert_eq!(10, leaf.data().wins());
    }

    #[test]
    fn backprop_saturation_expects_becomes_saturated_when_all_children_saturated() {
        let data = {
            let mut state = TicTacToeState::initial_state();

            // ___
            // ___
            // X__
            state.apply_move(TicTacToeAction::from_str("0,0").unwrap());

            // ___
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("1,1").unwrap());

            // __X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("2,2").unwrap());

            // O_X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("0,2").unwrap());

            // O_X
            // _O_
            // X_X
            state.apply_move(TicTacToeAction::from_str("2,0").unwrap());

            // O_X
            // OO_
            // X_X
            state.apply_move(TicTacToeAction::from_str("0,1").unwrap());

            // OXX
            // OO_
            // X_X
            state.apply_move(TicTacToeAction::from_str("1,2").unwrap());

            // OXX
            // OO_
            // XOX
            state.apply_move(TicTacToeAction::from_str("1,0").unwrap());

            MctsData::new(state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        expand(&tree_root).unwrap();
        let children = tree_root.children_read();
        let children = children.iter().cloned().collect::<Vec<_>>();

        assert!(
            !tree_root.data().is_saturated(),
            "Every child is saturated, but not every child has had its saturation status backpropagated, so the root should not be considered saturated."
        );

        // backprop the one remaining child.
        expand(children[0].borrow()).unwrap();
        backprop_saturation(children[0].borrow());

        assert!(
            tree_root.data().is_saturated(),
            "Now that every child has had its saturation backpropagated, the parent should be considered saturated as well."
        );
    }

    #[test]
    fn terminal_node_is_considered_saturated() {
        let data = {
            let mut state = TicTacToeState::initial_state();

            // ___
            // ___
            // X__
            state.apply_move(TicTacToeAction::from_str("0,0").unwrap());

            // ___
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("1,1").unwrap());

            // __X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("2,2").unwrap());

            // O_X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("0,2").unwrap());

            // O_X
            // _O_
            // X_X
            state.apply_move(TicTacToeAction::from_str("2,0").unwrap());

            // O_X
            // OO_
            // X_X
            state.apply_move(TicTacToeAction::from_str("0,1").unwrap());

            // OXX
            // OO_
            // X_X
            state.apply_move(TicTacToeAction::from_str("1,2").unwrap());

            // OXX
            // OO_
            // XOX
            state.apply_move(TicTacToeAction::from_str("1,0").unwrap());

            MctsData::new(state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        expand(&tree_root).unwrap();
        let children = tree_root.children_read();
        let children = children.iter().cloned().collect::<Vec<_>>();

        assert_eq!(
            1,
            children.len(),
            "Only one move left, so expect only 1 child."
        );
        assert!(
            !children[0].borrow().data().is_saturated(),
            "Not considered saturated, since we have not expanded yet (so we don't know for sure)"
        );

        expand(children[0].borrow()).unwrap();

        assert!(
            children[0].borrow().data().is_saturated(),
            "Now that we've expanded, we know it is saturated."
        );
    }

    #[test]
    fn backprop_saturation_expects_updates_worst_win_play_counts() {
        let data = {
            let mut state = TicTacToeState::initial_state();

            // ___
            // ___
            // X__
            state.apply_move(TicTacToeAction::from_str("0,0").unwrap());

            // ___
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("1,1").unwrap());

            // __X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("2,2").unwrap());

            // O_X
            // _O_
            // X__
            state.apply_move(TicTacToeAction::from_str("0,2").unwrap());

            MctsData::new(state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        mcts(&tree_root, PlayerColor::Black);

        assert!(
            tree_root.data().is_saturated(),
            "MCTS must have saturated the root, or this test is meaningless."
        );

        let children = tree_root.children_read();
        let children = children.iter().cloned().collect::<Vec<_>>();

        let loss_children = children.iter().filter(|&c| {
            c.borrow().data().action().unwrap() != TicTacToeAction::from_str("2,0").unwrap()
        });

        let _win_child = children.iter().filter(|&c| {
            c.borrow().data().action().unwrap() == TicTacToeAction::from_str("2,0").unwrap()
        });

        for loss_child in loss_children {
            let (wwins, wplays) = loss_child.borrow().data().worst_case_wins_plays();

            assert_eq!(0, wwins,
                "Worst case is alawys 0 wins, since these loss actions leave 2,0, open for white to win immediately.");

            assert_eq!(1, wplays,
                "Worst case is alawys 0 wins, since these loss actions leave 2,0, open for white to win immediately.");
        }
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    #[allow(clippy::float_cmp)]
    fn score_node_expects_always_prefers_univisted_node() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.new_child(data.clone());
        let child_b = tree_root.new_child(data.clone());
        let child_c = tree_root.new_child(data.clone());
        let child_d = tree_root.new_child(data.clone());

        // "visit" each child a different amount of times
        // child a: three visits
        let is_win = false;
        backprop_sim_result(child_a.borrow(), is_win);
        backprop_sim_result(child_a.borrow(), is_win);
        backprop_sim_result(child_a.borrow(), is_win);

        // child b: two visits
        backprop_sim_result(child_b.borrow(), is_win);
        backprop_sim_result(child_b.borrow(), is_win);

        // child c: one visit
        backprop_sim_result(child_c.borrow(), is_win);

        let parent_plays = tree_root.data().plays();

        assert_eq!(
            1.2365144,
            score_node_for_traversal(child_a.borrow(), parent_plays, true)
        );
        assert_eq!(
            1.5144148,
            score_node_for_traversal(child_b.borrow(), parent_plays, true)
        );
        assert_eq!(
            2.141706,
            score_node_for_traversal(child_c.borrow(), parent_plays, true)
        );
        assert_eq!(
            340282350000000000000000000000000000000f32,
            score_node_for_traversal(child_d.borrow(), parent_plays, true)
        );
    }

    #[test]
    fn simulate_runs_to_completion_and_terminates() {
        let mut initial_state = make_test_state();
        initial_state.initialize_board();
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        let _sim_result = simulate(&tree_root, &mut crate::util::get_rng_deterministic());
    }

    #[test]
    fn mcts_when_sufficient_resources_expects_saturates_root_node() {
        let mut state = TicTacToeState::new();

        // __X
        // _O_
        // X__
        let moves = vec!["0,0", "1,1", "2,2"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        assert!(
            !root.data().is_saturated(),
            "The node must not be saturated to begin with."
        );

        mcts(root, PlayerColor::Black);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated after sufficient MCTS traversal. (Is the test being run with an adequate amount of simulations?)"
        );
    }

    #[test]
    fn mcts_saturates_root_node() {
        let mut state = TicTacToeState::new();

        // XOX
        // OOX
        // X_O
        let moves = vec!["0,0", "1,1", "2,2", "1,2", "0,2", "0,1", "2,1", "2,0"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        assert!(
            !root.data().is_saturated(),
            "The node must not be saturated to begin with."
        );

        assert_eq!(
            PlayerColor::Black,
            root.data().state().current_player_turn()
        );

        mcts(root, PlayerColor::Black);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated after sufficient MCTS traversal. (Is the test being run with an adequate amount of simulations?)"
        );
    }

    #[test]
    fn mcts_expects_parent_play_count_sum_children_play_counts() {
        let mut state = TicTacToeState::new();

        // __X
        // _O_
        // X__
        let moves = vec!["0,0", "1,1", "2,2"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        mcts(root, PlayerColor::Black);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated for this test to be valid. (Is the test being run with an adequate amount of simulations?)"
        );

        let mut traversal = vec![root.get_handle()];
        while let Some(n) = traversal.pop() {
            let node: &ArcNode<_> = n.borrow();

            let node_play_count = node.data().plays();
            let child_play_sum: usize = node.children_read().iter().map(|c| c.data().plays()).sum();

            assert!(
                // Note: this is a bit of a hack right now, they should be exactly equal
                // but the root node is a special case that doesn't ever get played itself, only its children.
                node_play_count - child_play_sum <= 1,
                "A node's play count ({}) must be the sum of its children's play counts + 1 ({}) (because the parent itself is also played.)",
                node_play_count, child_play_sum
            );

            let children = node.children_read();
            let children = children.iter().cloned();
            traversal.extend(children);
        }
    }

    #[test]
    fn mcts_when_root_saturated_expects_all_terminals_played_exactly_once() {
        let mut state = TicTacToeState::new();

        // __X
        // _O_
        // X__
        let moves = vec!["0,0", "1,1", "2,2"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        mcts(root, PlayerColor::White);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated for this test to be valid. (Is the test being run with an adequate amount of simulations?)"
        );

        let mut traversal = vec![root.get_handle()];
        while let Some(n) = traversal.pop() {
            let node: &ArcNode<_> = n.borrow();

            if node.children_read().is_empty() {
                assert_eq!(
                    node.data().plays(),
                    1,
                    "A terminal node with no children must have been played exactly one time."
                );
            }

            let children = node.children_read();
            let children = children.iter().cloned();
            traversal.extend(children);
        }
    }

    #[test]
    fn mcts_when_root_saturated_expects_terminal_count_equals_terminal_count() {
        let mut state = TicTacToeState::new();

        // __X
        // _O_
        // X__
        let moves = vec!["0,0", "1,1", "2,2"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        mcts(root, PlayerColor::White);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated for this test to be valid. (Is the test being run with an adequate amount of simulations?)"
        );

        let mut terminal_count = 0;
        let mut traversal = vec![root.get_handle()];
        while let Some(n) = traversal.pop() {
            let node: &ArcNode<_> = n.borrow();

            if node.children_read().is_empty() {
                terminal_count += 1;
            }

            let children = node.children_read();
            let children = children.iter().cloned();
            traversal.extend(children);
        }

        assert_eq!(terminal_count, root.data().terminal_count(),
        "Expected the root's terminal count after saturation to equal the count of terminal's in the tree.");
    }

    fn black_isnt_stupid() {
        // In this board (black just played (3,1), white to play),
        // MCTS spent 80.06% of time simulating
        // white responding with (1,1),
        // but instead white responds with (6,3)
        // 7| - - X - - X - -
        // 6| - - X X X X - -
        // 5| X X X X X X X -
        // 4| X X O O O O X -
        // 3| O O X O O X - X
        // 2| X X X X X X X X
        // 1| - - X X X X - -
        // 0| - - X X O X - -
        //   ----------------
        //    0 1 2 3 4 5 6 7
        let board_str = "
            7| - - X - - X - -
            6| - - X X X X - -
            5| X X X X X X X -
            4| X X O O O O X -
            3| O O X O O X - X
            2| X X X X X X X X
            1| - - X X X X - -
            0| - - X X O X - -
              ----------------
               0 1 2 3 4 5 6 7
        ";
    }

    #[test]
    fn mcts_expects_final_saturation_increases_root_terminal_count() {
        let mut state = TicTacToeState::new();

        // XOX
        // OOX
        // X_O
        let moves = vec!["0,0", "1,1", "2,2", "1,2", "0,2", "0,1", "2,1", "2,0"]
            .into_iter()
            .map(|s| TicTacToeAction::from_str(s).unwrap());

        state.apply_moves(moves);

        let root_handle = ArcNode::new_root(MctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        assert!(
            !root.data().is_saturated(),
            "The node must not be saturated to begin with."
        );

        let root_terminal_count_before = root.data().terminal_count();

        assert_eq!(
            PlayerColor::Black,
            root.data().state().current_player_turn()
        );

        mcts(root, PlayerColor::Black);

        let root_terminal_count_after = root.data().terminal_count();

        assert_eq!(
            root_terminal_count_after, root_terminal_count_before + 1,
            "By adding one new saturated node, expects root to get its terminal count incremented by one."
        );
    }
}
