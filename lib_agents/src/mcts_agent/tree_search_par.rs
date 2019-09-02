use crate::util;

use lib_boardgame::GameResult;
use lib_boardgame::{GameState, PlayerColor};
use monte_carlo_tree::dot_visualize::TreeToDotFileFormat;
use monte_carlo_tree::{
    monte_carlo_data::MctsResult,
    amonte_carlo_data::AMctsData,
    tree::Node,
};
use std::borrow::Borrow;
use std::time::{Duration, Instant};
use crossbeam::thread;
use std::marker::{Sync, Send};
use std::clone::Clone;

// todo: mcts() should return the actual winning node,
// and if the subtree from the root is saturated
// it should use ratio of wins/plays inatead of sum(plays)
// as the score.

pub(super) const SIM_TIME_MS: u64 = 3_000;
const EXTRA_TIME_MS: u64 = 3_000;

fn expand<TNode, TState>(node: &TNode) -> Option<TNode::ChildrenIter>
where
    TNode: Node<Data = AMctsData<TState>>,
    TState: GameState,
{
    node.data().mark_expanded();

    let state = node.data().state();
    if state.is_game_over() {
        // if the game is over, we have nothing to expand
        node.data().set_children_count(0);
        return None;
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
        let _child_node = node.new_child(data);
    }

    Some(node.children())
}

/// Increment this node's count of saturated children.
/// If doing so results in this node itself becoming saturated,
/// follow the same operation for its parent.
fn backprop_saturation<TNode, TState>(leaf: &TNode)
where
    TNode: Node<Data = AMctsData<TState>>,
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
    TNode: Node<Data = AMctsData<TState>>,
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
    TNode: Node<Data = AMctsData<TState>>,
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
fn select_to_leaf_inverted<TNode, TState>(root: &TNode, player_color: PlayerColor) -> TNode::Handle
where
    TNode: Node<Data = AMctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child =
            select_child_max_score_inverted::<TNode, TState>(cur_node.borrow(), player_color);

        match selected_child {
            Some(c) => cur_node = c,
            None => return cur_node,
        }
    }
}

fn select_child_max_score_inverted<TNode, TState>(
    root: &TNode,
    player_color: PlayerColor,
) -> Option<TNode::Handle>
where
    TNode: Node<Data = AMctsData<TState>>,
    TState: GameState,
{
    // TODO: If the only play is "pass turn", then even if parent color is enemy, don't be pessimistic
    // since being forced to pass a turn is very bad for the enemy and good for the player
    let parent_is_player_color = root.borrow().data().state().current_player_turn() == player_color;
    let child_nodes = root.children();

    child_nodes
        .into_iter()
        .filter(|n| !n.borrow().data().is_saturated())
        .max_by(|a, b| {
            let a_score = score_node_pessimistic(a.borrow(), parent_is_player_color);
            let b_score = score_node_pessimistic(b.borrow(), parent_is_player_color);

            a_score.partial_cmp(&b_score).unwrap()
        })
}

fn score_node_pessimistic<TNode, TState>(node: &TNode, parent_is_player_color: bool) -> f32
where
    TNode: Node<Data = AMctsData<TState>>,
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

    let parent_plays = node.parent().map_or(0, |p| p.borrow().data().plays()) as f32;
    let bias = f32::sqrt(2_f32);

    (wins / plays) + bias * f32::sqrt(f32::ln(parent_plays) / plays)
}

pub fn mcts_result<TNode, TState>(
    state: TState,
    player_color: PlayerColor,
) -> Vec<MctsResult<TState>>
where
    TNode: Node<Data = AMctsData<TState>> + Sync,
    TState: GameState,
{
    let root_handle = TNode::new_root(AMctsData::new(state, 0, 0, None));
    let root = root_handle.borrow();

    mcts(root, player_color);

    let mut state_children = root.children().into_iter().collect::<Vec<_>>();

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

fn mcts<TNode, TState>(root: &TNode, player_color: PlayerColor)
where
    TNode: Node<Data = AMctsData<TState>> + Sync,
    TState: GameState,
{
    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|_| {
                mcts_loop(root, player_color);
            });
        }
    }).unwrap();
}

fn mcts_loop<TNode, TState>(root: &TNode, player_color: PlayerColor)
where
    TNode: Node<Data = AMctsData<TState>> + Sync,
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

        // If the leaf node had no possible children (i.e. it was also a terminating node)
        // then we should do our saturation backpropagation.
        if expanded_children.is_none() {
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
        let newly_expanded_children = expanded_children.unwrap().into_iter().collect::<Vec<_>>();

        let sim_node = util::random_pick(&newly_expanded_children, &mut rng)
            .expect("Must have had at least one expanded child.");
        let sim_node = sim_node.borrow();

        // simulate
        let sim_result = simulate(sim_node, &mut rng);

        // backprop
        let is_win = sim_result.is_win_for_player(player_color);
        backprop_sim_result(sim_node, is_win);
    }

    dbg!(sim_count);
}