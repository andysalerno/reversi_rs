use crate::util;

use crossbeam::thread;
use lib_boardgame::GameResult;
use lib_boardgame::{GameState, PlayerColor};
use monte_carlo_tree::{amonte_carlo_data::AMctsData, atree::ANode, monte_carlo_data::MctsResult};
use std::borrow::Borrow;
use std::time::{Duration, Instant};

// todo: mcts() should return the actual winning node,
// and if the subtree from the root is saturated
// it should use ratio of wins/plays inatead of sum(plays)
// as the score.

pub(super) const SIM_TIME_MS: u64 = 3_000;
const EXTRA_TIME_MS: u64 = 0_000;

fn expand<TNode, TState>(node: &TNode) -> Vec<TNode::Handle>
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    // Acquire the write lock on the children
    let mut children_write_lock = node.children_lock_write();

    if node.data().is_expanded() {
        // Another thread beat us to the punch, so no work to do
        drop(children_write_lock);
        return node.children_handles();
    }

    node.data().mark_expanded();

    let state = node.data().state();
    if state.is_game_over() {
        // if the game is over, we have nothing to expand
        node.data().set_children_count(0);
        return vec![];
    }

    // TODO: There's no reason for legal_moves() to need this argument
    // since the state already knows the player's turn.
    let player_turn = state.current_player_turn();
    let legal_actions = state.legal_moves(player_turn);

    // Now that we've expanded this node, update it to
    // inform it how many children it has.
    node.data().set_children_count(legal_actions.len());

    // create a new child node for every available action->state transition
    for &action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = AMctsData::new(resulting_state, 0, 0, Some(action));
        let _child_node = node.new_child(data, &mut children_write_lock);
    }

    (*children_write_lock).clone()
}

/// Increment this node's count of saturated children.
/// If doing so results in this node itself becoming saturated,
/// follow the same operation for its parent.
fn backprop_saturation<TNode, TState>(leaf: &TNode)
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    assert!(
        leaf.data().is_saturated(),
        "Only a leaf considered saturated can have its saturated status backpropagated."
    );
    let mut handle = leaf.parent();

    while let Some(p) = handle {
        let node = p.borrow();
        let data = node.data();
        data.increment_saturated_children_count();

        if !data.is_saturated() {
            // Don't back-prop any further
            // if we've reached a non-saturated node.
            return;
        }

        handle = node.parent();
    }
}

fn simulate<TNode, TState, R>(node: &TNode, rng: &mut R) -> GameResult
where
    TNode: ANode<Data = AMctsData<TState>>,
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

fn backprop_sim_result<TNode, TState>(node: &TNode, is_win: bool)
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let mut parent_node = Some(node.get_handle());

    while let Some(p) = parent_node {
        let parent = p.borrow();
        let data = parent.data();
        data.increment_plays();

        if is_win {
            data.increment_wins();
        }

        parent_node = parent.parent();
    }
}

/// Selects using max UCB, but on opponent's turn inverts the score.
/// If the given node has no children, returns a handle back to the given node.
fn select_to_leaf_inverted<TNode, TState>(root: &TNode, player_color: PlayerColor) -> TNode::Handle
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    while let Some(c) =
        select_child_max_score_inverted::<TNode, TState>(cur_node.borrow(), player_color)
    {
        cur_node = c;
    }

    cur_node
}

/// Returns a handle to the child with the greatest selection score,
/// or None if there are no children OR all children have been saturated.
fn select_child_max_score_inverted<TNode, TState>(
    root: &TNode,
    player_color: PlayerColor,
) -> Option<TNode::Handle>
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let parent_data = root.data();
    let parent_is_player_color = parent_data.state().current_player_turn() == player_color;
    let parent_plays = parent_data.plays();
    let parent_plays = usize::max(1, parent_plays);

    let child_nodes = root.children_lock_read();

    (*child_nodes)
        .iter()
        .filter(|&n| !n.borrow().data().is_saturated())
        .max_by(|&a, &b| {
            let a_score = score_node_pessimistic(a.borrow(), parent_plays, parent_is_player_color);
            let b_score = score_node_pessimistic(b.borrow(), parent_plays, parent_is_player_color);

            a_score.partial_cmp(&b_score).unwrap()
        })
        .and_then(|n| Some(n.clone()))
}

fn score_node_pessimistic<TNode, TState>(
    node: &TNode,
    parent_plays: usize,
    parent_is_player_color: bool,
) -> f32
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let data = node.data();
    let plays = data.plays() as f32;

    if plays == 0f32 {
        return std::f32::MAX;
    }

    let wins = if parent_is_player_color {
        data.wins() as f32
    } else {
        debug_assert!(data.plays() >= data.wins());
        (data.plays() - data.wins()) as f32
    };

    let parent_plays = parent_plays as f32;

    let node_mean_val = wins / plays;
    let explore_bias = 2_f32;

    // todo: test swapping parent_plays and plays, and run trials for that
    let score = node_mean_val + f32::sqrt((explore_bias * f32::ln(parent_plays)) / plays);

    if score.is_nan() {
        panic!(
            "plays: {}\nwins: {}\nparent_plays: {}\nparent_is_player_color: {}",
            plays, wins, parent_plays, parent_is_player_color
        );
    }

    score
}

pub fn mcts_result<TNode, TState>(
    state: TState,
    player_color: PlayerColor,
    thread_count: usize,
) -> Vec<MctsResult<TState>>
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let root_handle = TNode::new_root(AMctsData::new(state, 0, 0, None));
    let root = root_handle.borrow();

    mcts(root, player_color, thread_count);

    let mut state_children = root.children_handles();

    if root.data().is_saturated() {
        state_children
            .sort_by_key(|c| (c.borrow().data().wins() * 1000) / c.borrow().data().plays());
    } else {
        state_children.sort_by_key(|c| c.borrow().data().plays());
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

fn mcts<TNode, TState>(root: &TNode, player_color: PlayerColor, thread_count: usize)
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    if thread_count == 1 {
        mcts_loop(root, player_color);
    } else {
        thread::scope(|s| {
            for _ in 0..thread_count {
                s.spawn(|_| {
                    mcts_loop(root, player_color);
                });
            }
        })
        .unwrap();
    }
}

fn mcts_loop<TNode, TState>(root: &TNode, player_color: PlayerColor)
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState,
{
    let now = Instant::now();
    let exec_duration = Duration::from_millis(SIM_TIME_MS);
    let extra_time = Duration::from_millis(EXTRA_TIME_MS);

    let mut rng = util::get_rng();

    let mut sim_count: usize = 0;

    loop {
        if now.elapsed() >= exec_duration {
            let data = root.data();

            if (data.wins() * 1000) / data.plays() > 500
                || now.elapsed() >= exec_duration + extra_time
            {
                break;
            }
        }

        sim_count += 1;
        // If we have completely explored this entire tree,
        // there's nothing left to do.
        if root.data().is_saturated() {
            break;
        }

        // Select: travel down to a leaf node, using the explore/exploit rules.
        let leaf = select_to_leaf_inverted(root, player_color);

        let leaf = leaf.borrow();

        // Expand: generate fresh child nodes for the selected leaf node.
        let expanded_children = expand(leaf);

        if !expanded_children.is_empty() {
            let sim_node = util::random_pick(&expanded_children, &mut rng)
                .expect("Must have had at least one expanded child.");
            let sim_node = sim_node.borrow();

            // simulate
            let sim_result = simulate(sim_node, &mut rng);

            // backprop
            let is_win = sim_result.is_win_for_player(player_color);
            backprop_sim_result(sim_node, is_win);
        } else {
            // We expanded the node, but it had no children,
            // so this node must be a terminating node.
            let sim_result = simulate(leaf, &mut rng);

            if leaf.data().plays() == 0 {
                let is_win = sim_result.is_win_for_player(player_color);
                backprop_sim_result(leaf, is_win);
            }

            // Update the terminating node so it knows its own end game result.
            leaf.data().set_end_state_result(sim_result);

            backprop_saturation(leaf);

            continue;
        }
    }

    dbg!(sim_count);
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

    fn make_node<G>(data: AMctsData<G>) -> impl ANode<Data = AMctsData<G>>
    where
        G: GameState + Sync,
        G::Move: Sync,
    {
        ArcNode::new_root(data)
    }

    fn make_test_data() -> AMctsData<TicTacToeState> {
        AMctsData::new(TicTacToeState::initial_state(), 0, 0, None)
    }

    #[test]
    fn new_child_expects_add_child_to_parent() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());
        let child = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());

        assert_eq!(1, tree_root.children_handles().len());
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
        let child_level_1 = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_level_2 = child_level_1.borrow().new_child(
            data.clone(),
            &mut child_level_1.borrow().children_lock_write(),
        );
        let child_level_3 = child_level_2.borrow().new_child(
            data.clone(),
            &mut child_level_2.borrow().children_lock_write(),
        );
        let child_level_4 = child_level_3.borrow().new_child(
            data.clone(),
            &mut child_level_3.borrow().children_lock_write(),
        );

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

        let expanded_children = expand(&tree_root).into_iter().collect::<Vec<_>>();

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, expanded_children.len());
    }

    #[test]
    fn expand_expects_adds_children_to_parent() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.children_handles().len());

        expand(&tree_root);

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, tree_root.children_handles().len());
    }

    #[test]
    fn expand_expects_marks_node_expanded() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert!(!tree_root.data().is_expanded());

        expand(&tree_root);

        assert!(tree_root.data().is_expanded());
    }

    #[test]
    fn expand_expects_updates_children_count() {
        let tree_root = ArcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.data().children_count());

        expand(&tree_root);

        assert_eq!(9, tree_root.data().children_count());
    }

    #[test]
    fn select_child_max_score_expects_picks_less_explored_node() {
        let data = AMctsData::new(TicTacToeState::new(), 0, 0, None);

        let tree_root = ArcNode::new_root(data.clone());

        let child_level_1 = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_level_1: &ArcNode<_> = child_level_1.borrow();

        let child_level_2 =
            child_level_1.new_child(data.clone(), &mut child_level_1.children_lock_write());
        let child_level_2: &ArcNode<_> = child_level_2.borrow();

        let child_level_3_handle =
            child_level_2.new_child(data.clone(), &mut child_level_2.children_lock_write());
        let child_level_3: &ArcNode<_> = child_level_3_handle.borrow();

        let child_level_4 =
            child_level_3.new_child(data.clone(), &mut child_level_3.children_lock_write());
        let child_level_4: &ArcNode<_> = child_level_4.borrow();

        let child_level_4b =
            child_level_3.new_child(data.clone(), &mut child_level_3.children_lock_write());
        let child_level_4b: &ArcNode<_> = child_level_4b.borrow();

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

        let selected = select_child_max_score_inverted::<ArcNode<_>, TicTacToeState>(
            &child_level_3_handle,
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
        let child_level_1 = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_level_2 = child_level_1.borrow().new_child(
            data.clone(),
            &mut child_level_1.borrow().children_lock_write(),
        );
        let child_level_3 = child_level_2.borrow().new_child(
            data.clone(),
            &mut child_level_2.borrow().children_lock_write(),
        );
        let child_level_4 = child_level_3.borrow().new_child(
            data.clone(),
            &mut child_level_3.borrow().children_lock_write(),
        );
        let child_level_4b = child_level_3.borrow().new_child(
            data.clone(),
            &mut child_level_3.borrow().children_lock_write(),
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

        let leaf = select_to_leaf_inverted(&tree_root, PlayerColor::Black);

        let leaf = leaf.borrow();

        assert_eq!(2, leaf.data().plays());
    }

    #[test]
    fn select_to_leaf_expects_when_already_leaf_returns_self() {
        let data = AMctsData::new(TicTacToeState::initial_state(), 10, 10, None);

        let tree_root = make_node(data.clone());

        let leaf = select_to_leaf_inverted(&tree_root, PlayerColor::Black);
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

            AMctsData::new(state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        let children = expand(tree_root.borrow()).into_iter().collect::<Vec<_>>();

        assert!(
            !tree_root.data().is_saturated(),
            "Every child is saturated, but not every child has had its saturation status backpropagated, so the root should not be considered saturated."
        );

        // backprop the one remaining child.
        expand(children[0].borrow());
        backprop_saturation(children[0].borrow());

        assert!(
            tree_root.data().is_saturated(),
            "Now that every child has had its saturation backpropagated, the parent should be considered saturated as well."
        );
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    #[allow(clippy::float_cmp)]
    fn score_node_expects_always_prefers_univisted_node() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_b = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_c = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());
        let child_d = tree_root.new_child(data.clone(), &mut tree_root.children_lock_write());

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
            1.0929347,
            score_node_pessimistic(child_a.borrow(), parent_plays, true)
        );
        assert_eq!(
            1.3385662,
            score_node_pessimistic(child_b.borrow(), parent_plays, true)
        );
        assert_eq!(
            1.8930185,
            score_node_pessimistic(child_c.borrow(), parent_plays, true)
        );
        assert_eq!(
            340282350000000000000000000000000000000f32,
            score_node_pessimistic(child_d.borrow(), parent_plays, true)
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

        let root_handle = ArcNode::new_root(AMctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        assert!(
            !root.data().is_saturated(),
            "The node must not be saturated to begin with."
        );

        mcts(root, PlayerColor::Black, 1);

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

        let root_handle = ArcNode::new_root(AMctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        mcts(root, PlayerColor::Black, 1);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated for this test to be valid. (Is the test being run with an adequate amount of simulations?)"
        );

        let mut traversal = vec![root.get_handle()];
        while let Some(n) = traversal.pop() {
            let node: &ArcNode<_> = n.borrow();

            let node_play_count = node.data().plays();
            let child_play_sum: usize = node
                .children_handles()
                .into_iter()
                .map(|c| c.data().plays())
                .sum();

            assert!(
                // Note: this is a bit of a hack right now, they should be exactly equal
                // but the root node is a special case that doesn't ever get played itself, only its children.
                node_play_count - child_play_sum <= 1,
                "A node's play count (left) must be the sum of its children's play counts + 1 (right) (because the parent itself is also played.)"
            );

            traversal.extend(node.children_handles());
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

        let root_handle = ArcNode::new_root(AMctsData::new(state, 0, 0, None));
        let root: &ArcNode<_> = root_handle.borrow();

        mcts(root, PlayerColor::Black, 1);

        assert!(
            root.data().is_saturated(),
            "The node must become saturated for this test to be valid. (Is the test being run with an adequate amount of simulations?)"
        );

        let mut traversal = vec![root.get_handle()];
        while let Some(n) = traversal.pop() {
            let node: &ArcNode<_> = n.borrow();

            if node.children_handles().is_empty() {
                assert_eq!(
                    node.data().plays(),
                    1,
                    "A terminal node with no children must have been played exactly one time."
                );
            }

            traversal.extend(node.children_handles());
        }
    }
}
