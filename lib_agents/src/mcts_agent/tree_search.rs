use crate::mcts_agent::mcts_data::{Data, MctsData, MctsResult};
use crate::util;

use lib_boardgame::GameResult;
use lib_boardgame::{GameState, PlayerColor};
use monte_carlo_tree::Node;
use std::borrow::Borrow;

pub(super) const TOTAL_SIMS: usize = 500;

fn expand<TNode, TState>(node: &TNode) -> Option<TNode::ChildrenIter>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    node.data().mark_expanded();

    // todo: unnecessary optimization here?
    {
        let children_len = node.children().into_iter().count();
        assert_eq!(0, children_len);
    }

    let state = node.data().state();
    if state.is_game_over() {
        // if the game is over, we have nothing to expand
        return None;
    }

    let player_turn = state.current_player_turn();
    let legal_actions = state.legal_moves(player_turn);

    // Now that we've expanded this node, update it to
    // inform it how many children it has.
    node.data().set_children_count(legal_actions.len());

    // create a new child node for every available action->state transition
    for action in legal_actions {
        let resulting_state = state.next_state(action);
        let data = MctsData::new(&resulting_state, 0, 0, Some(action));
        let _child_node = node.new_child(data);
    }

    Some(node.children())
}

/// Increment this node's count of saturated children.
/// If doing so results in this node itself becoming saturated,
/// follow the same operation for its parent.
fn backprop_saturation<TNode, TState>(leaf: &TNode)
where
    TNode: Node<Data = MctsData<TState>>,
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

fn backprop_sim_result<TNode, TState>(node: &TNode, result: GameResult, color: PlayerColor)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let incr_wins = result.is_win_for_player(color);

    let mut parent_node = Some(node.get_handle());

    while let Some(p) = parent_node {
        let parent = p.borrow();
        let data = parent.data();
        data.increment_plays();

        if incr_wins {
            data.increment_wins();
        }
        parent_node = parent.parent();
    }
}

/// Always chooses to select the child with the best win/plays ratio,
/// even on the opponent's turn (i.e. no pessimism).
#[allow(unused)]
fn select_to_leaf_uninverted<TNode, TState>(
    root: &TNode,
    player_color: PlayerColor,
) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child = select_child_max_score::<TNode, TState>(cur_node.clone());

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

/// Selects using max UCB, but on opponent's turn picks randomly.
#[allow(unused)]
fn select_to_leaf_rand<TNode, TState, Rng>(
    root: &TNode,
    player_color: PlayerColor,
    rng: &mut Rng,
) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    Rng: rand::Rng,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child =
            if player_color == cur_node.borrow().data().state().current_player_turn() {
                select_child_max_score::<TNode, TState>(cur_node.clone())
            } else {
                select_child_rand::<TNode, TState, _>(cur_node.clone(), rng)
            };

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

/// Selects using max UCB, but on opponent's turn inverts the score.
#[allow(unused)]
fn select_to_leaf_inverted<TNode, TState>(root: &TNode, player_color: PlayerColor) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child =
            if player_color == cur_node.borrow().data().state().current_player_turn() {
                select_child_max_score::<TNode, TState>(cur_node.clone())
            } else {
                select_child_max_score_inverted::<TNode, TState>(cur_node.clone(), player_color)
            };

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

/// For all children of the given node, assign each one a score,
/// and return the child with the highest score (ties broken by the first)
/// or None if there are no unsaturated children.
fn select_child_max_score<TNode, TState>(root: TNode::Handle) -> Option<TNode::Handle>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let child_nodes = root.borrow().children();

    child_nodes
        .into_iter()
        .filter(|n| !n.borrow().data().is_saturated())
        .max_by(|a, b| {
            let a_score = score_node_simple(a.borrow());
            let b_score = score_node_simple(b.borrow());

            a_score.partial_cmp(&b_score).unwrap()
        })
}

#[allow(unused)]
fn select_child_max_score_inverted<TNode, TState>(
    root: TNode::Handle,
    player_color: PlayerColor,
) -> Option<TNode::Handle>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let child_nodes = root.borrow().children();

    child_nodes
        .into_iter()
        .filter(|n| !n.borrow().data().is_saturated())
        .max_by(|a, b| {
            let a_score = score_node_pessimistic(a.borrow(), player_color);
            let b_score = score_node_pessimistic(b.borrow(), player_color);

            a_score.partial_cmp(&b_score).unwrap()
        })
}

#[allow(unused)]
fn select_child_rand<TNode, TState, Rng>(
    root: TNode::Handle,
    rng: &mut Rng,
) -> Option<TNode::Handle>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    Rng: rand::Rng,
{
    let child_nodes = root.borrow().children();

    let unsaturated_children = child_nodes
        .into_iter()
        .filter(|n| !n.borrow().data().is_saturated())
        .collect::<Vec<_>>();

    let selected_child = util::random_pick(&unsaturated_children, rng);
    selected_child.cloned()
}

/// Given a node, score it in such a way that encourages
/// both exploration and exploitation of the state space.
fn score_node_simple<TNode, TState>(node: &TNode) -> f32
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    // score = win_ratio + sqrt( 2lg(parent_plays) / plays)

    let plays = node.data().plays() as f32;

    if plays == 0f32 {
        return std::f32::MAX;
    }

    let wins = node.data().wins() as f32;
    let parent_plays = node.parent().map_or(0, |p| p.borrow().data().plays()) as f32;
    let bias = 2_f32;

    (wins / plays) + f32::sqrt((bias * f32::ln(parent_plays)) / plays)
}

#[allow(unused)]
fn score_node_pessimistic<TNode, TState>(node: &TNode, player_color: PlayerColor) -> f32
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let plays = node.data().plays() as f32;

    if plays == 0f32 {
        return std::f32::MAX;
    }

    let wins = if player_color == node.data().state().current_player_turn() {
        node.data().wins() as f32
    } else {
        (node.data().plays() - node.data().wins()) as f32
    };

    let parent_plays = node.parent().map_or(0, |p| p.borrow().data().plays()) as f32;
    let bias = 2_f32;

    (wins / plays) + f32::sqrt((bias * f32::ln(parent_plays)) / plays)
}

/// Given a node, score it by giving it a value we can use
/// to rank which node should be returned by this agent
/// as the node to play in the game.
/// This has outperformed the _plays version.
pub(super) fn score_mcts_results_ratio<TNode, TState>(
    mcts_result: &MctsResult<TState>,
    color: PlayerColor,
) -> usize
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let is_win = if let Some(game_result) = mcts_result.result {
        game_result.is_win_for_player(color)
    } else {
        false
    };

    if is_win {
        return std::usize::MAX;
    }

    if mcts_result.plays == 0 {
        0
    } else {
        let ratio = (mcts_result.wins * 100) / mcts_result.plays;
        ratio as usize
    }
}

#[allow(unused)]
pub(super) fn score_mcts_results_plays<TNode, TState>(
    mcts_result: &MctsResult<TState>,
    color: PlayerColor,
) -> usize
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let is_win = if let Some(game_result) = mcts_result.result {
        (game_result == GameResult::BlackWins && color == PlayerColor::Black)
            || (game_result == GameResult::WhiteWins && color == PlayerColor::White)
    } else {
        false
    };

    if is_win {
        return std::usize::MAX;
    }

    mcts_result.plays
}

pub fn mcts<TNode, TState, Rng>(
    state: TState,
    player_color: PlayerColor,
    rng: &mut Rng,
) -> Vec<MctsResult<TState>>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    Rng: rand::Rng,
{
    let root_handle = TNode::new_root(MctsData::new(&state, 0, 0, None));
    let root = root_handle.borrow();

    for _ in 0..TOTAL_SIMS {
        // If we have completely explored this entire tree,
        // there's nothing left to do.
        if root.data().is_saturated() {
            break;
        }

        // Select: travel down to a leaf node, using the explore/exploit rules.
        let leaf = select_to_leaf_uninverted::<TNode, TState>(root, player_color);
        let leaf = leaf.borrow();

        // Expand: generate fresh child nodes for the selected leaf node.
        let expanded_children = expand(leaf);

        // If the leaf node had no possible children (i.e. it was also a terminating node)
        // then we should do our saturation backpropagation.
        if expanded_children.is_none() {
            let sim_result = simulate(leaf, rng);

            if leaf.data().plays() == 0 {
                backprop_sim_result(leaf, sim_result, player_color);
            }

            // Update the terminating node so it knows its own end game result.
            leaf.data().set_end_state_result(sim_result);

            backprop_saturation(leaf);

            continue;
        }

        let newly_expanded_children = expanded_children.unwrap().into_iter().collect::<Vec<_>>();

        let sim_node = util::random_pick(&newly_expanded_children, rng)
            .expect("Must have had at least one expanded child.");
        let sim_node = sim_node.borrow();

        // simulate
        let sim_result = simulate(sim_node, rng);

        // backprop
        backprop_sim_result(sim_node, sim_result, player_color);
    }

    let state_children = root.children();

    state_children
        .into_iter()
        .map(|c| c.borrow().data().into())
        .collect::<Vec<_>>()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use lib_tic_tac_toe::{
        tic_tac_toe::TicTacToe,
        tic_tac_toe_gamestate::{BoardPosition, TicTacToeAction, TicTacToeState},
        TicTacToePiece,
    };

    use std::str::FromStr;

    use monte_carlo_tree::rc_tree::RcNode;

    fn make_test_state() -> impl GameState {
        TicTacToeState::new()
    }

    fn make_node<G: GameState>(data: MctsData<G>) -> impl Node<Data = MctsData<G>> {
        RcNode::new_root(data)
    }

    fn make_test_data() -> MctsData<impl GameState> {
        MctsData::new(&make_test_state(), 0, 0, None)
    }

    #[test]
    fn new_child_expects_add_child_to_parent() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());
        let child = tree_root.new_child(data.clone());

        assert_eq!(1, tree_root.children().into_iter().count());
        assert!(child.borrow().parent().is_some());
        assert!(tree_root.parent().is_none());
    }

    #[test]
    fn backprop_sim_results_when_black_wins_expects_update_plays_wins() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::BlackWins, PlayerColor::Black);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(1, tree_root.data().wins());
    }

    #[test]
    fn backprop_sim_results_when_white_wins_expects_update_plays_wins() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::WhiteWins, PlayerColor::White);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(1, tree_root.data().wins());
    }

    #[test]
    fn backprop_sim_results_when_black_wins_expects_update_white_plays_not_wins() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::BlackWins, PlayerColor::White);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, tree_root.data().wins());
    }

    #[test]
    fn backprop_sim_results_expects_tie_game_is_not_a_win() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::Tie, PlayerColor::White);

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

        backprop_sim_result(
            child_level_3.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );

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
        let tree_root = RcNode::new_root(make_test_data());

        let expanded_children = expand(&tree_root)
            .expect("must have children")
            .into_iter()
            .collect::<Vec<_>>();

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, expanded_children.len());
    }

    #[test]
    fn expand_expects_adds_children_to_parent() {
        let tree_root = RcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.children().into_iter().count());

        expand(&tree_root);

        // The game used for testing is TicTacToe,
        // which has nine intitial legal children positions.
        assert_eq!(9, tree_root.children().into_iter().count());
    }

    #[test]
    fn expand_expects_marks_node_expanded() {
        let tree_root = RcNode::new_root(make_test_data());

        assert!(!tree_root.data().is_expanded());

        expand(&tree_root);

        assert!(tree_root.data().is_expanded());
    }

    #[test]
    fn expand_expects_updates_children_count() {
        let tree_root = RcNode::new_root(make_test_data());

        assert_eq!(0, tree_root.data().children_count());

        expand(&tree_root);

        assert_eq!(9, tree_root.data().children_count());
    }

    #[test]
    fn select_child_max_score_expects_picks_less_explored_node() {
        let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);

        let tree_root = RcNode::new_root(data.clone());

        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_1: &RcNode<_> = child_level_1.borrow();

        let child_level_2 = child_level_1.new_child(data.clone());
        let child_level_2: &RcNode<_> = child_level_2.borrow();

        let child_level_3_handle = child_level_2.new_child(data.clone());
        let child_level_3: &RcNode<_> = child_level_3_handle.borrow();

        let child_level_4 = child_level_3.new_child(data.clone());
        let child_level_4: &RcNode<_> = child_level_4.borrow();

        let child_level_4b = child_level_3.new_child(data.clone());
        let child_level_4b: &RcNode<_> = child_level_4b.borrow();

        child_level_1.data().set_children_count(1);
        child_level_2.data().set_children_count(1);
        child_level_3.data().set_children_count(2);
        child_level_4.data().set_children_count(1);
        child_level_4b.data().set_children_count(1);

        backprop_sim_result(child_level_3, GameResult::BlackWins, PlayerColor::Black);
        backprop_sim_result(child_level_4, GameResult::BlackWins, PlayerColor::Black);
        backprop_sim_result(child_level_4, GameResult::BlackWins, PlayerColor::Black);
        backprop_sim_result(child_level_4, GameResult::BlackWins, PlayerColor::Black);
        backprop_sim_result(child_level_4b, GameResult::BlackWins, PlayerColor::Black);

        assert!(!child_level_3.data().is_saturated());

        let selected = select_child_max_score::<RcNode<_>, TicTacToeState>(child_level_3_handle)
            .expect("the child should have been selected.");

        let selected: &RcNode<_> = selected.borrow();

        assert_eq!(1, selected.data().plays());
    }

    #[test]
    fn select_to_leaf_expects_selects_less_explored_path() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());
        let child_level_1 = tree_root.new_child(data.clone());
        let child_level_2 = child_level_1.borrow().new_child(data.clone());
        let child_level_3 = child_level_2.borrow().new_child(data.clone());
        let child_level_4 = child_level_3.borrow().new_child(data.clone());
        let child_level_4b = child_level_3.borrow().new_child(data.clone());

        tree_root.data().set_children_count(1);
        child_level_1.borrow().data().set_children_count(1);
        child_level_2.borrow().data().set_children_count(1);
        child_level_3.borrow().data().set_children_count(2);
        child_level_4.borrow().data().set_children_count(2);
        child_level_4b.borrow().data().set_children_count(2);

        backprop_sim_result(
            child_level_3.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4b.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );
        backprop_sim_result(
            child_level_4b.borrow(),
            GameResult::BlackWins,
            PlayerColor::Black,
        );

        let leaf = select_to_leaf_uninverted(&tree_root, PlayerColor::Black);

        let leaf = leaf.borrow();

        assert_eq!(2, leaf.data().plays());
    }

    #[test]
    fn select_to_leaf_expects_when_already_leaf_returns_self() {
        let data = MctsData::new(&make_test_state(), 10, 10, None);

        let tree_root = make_node(data.clone());

        let leaf = select_to_leaf_uninverted(&tree_root, PlayerColor::Black);
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

            MctsData::new(&state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        let children = expand(tree_root.borrow())
            .expect("must have children")
            .into_iter()
            .collect::<Vec<_>>();

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
    fn score_node_expects_always_prefers_univisted_node() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        let player_agent_color = PlayerColor::White;

        // all children of the same parent
        let child_a = tree_root.new_child(data.clone());
        let child_b = tree_root.new_child(data.clone());
        let child_c = tree_root.new_child(data.clone());
        let child_d = tree_root.new_child(data.clone());

        // "visit" each child a different amount of times
        // child a: three visits
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color);
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color);
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color);

        // child b: two visits
        backprop_sim_result(child_b.borrow(), GameResult::BlackWins, player_agent_color);
        backprop_sim_result(child_b.borrow(), GameResult::BlackWins, player_agent_color);

        // child c: one visit
        backprop_sim_result(child_c.borrow(), GameResult::BlackWins, player_agent_color);

        assert_eq!(1.0929347, score_node_simple(child_a.borrow()));
        assert_eq!(1.3385662, score_node_simple(child_b.borrow()));
        assert_eq!(1.8930185, score_node_simple(child_c.borrow()));
        assert_eq!(
            340282350000000000000000000000000000000f32,
            score_node_simple(child_d.borrow())
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
    pub fn mcts_score_results_ratio_expects_always_avoids_losing_move() {
        type BP = BoardPosition;
        // _ _ _
        // _ _ _
        // _ _ _
        let mut state = TicTacToeState::initial_state();

        // X _ _
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(0, 2)));

        // X _ O
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(2, 2)));

        // X _ O
        // _ _ _
        // X _ _
        state.apply_move(TicTacToeAction(BP::new(0, 0)));

        // White MUST block, or it will lose
        let mcts_results = mcts::<RcNode<_>, _, _>(
            state,
            PlayerColor::White,
            &mut util::get_rng_deterministic(),
        );

        let max_by_ratio = mcts_results
            .iter()
            .max_by_key(|c| score_mcts_results_ratio::<RcNode<_>, _>(c, PlayerColor::White))
            .unwrap();

        let expected_action = TicTacToeAction(BP::new(0, 1));
        assert_eq!(expected_action, max_by_ratio.action);
    }

    #[test]
    pub fn mcts_score_results_plays_expects_always_avoids_losing_move() {
        type BP = BoardPosition;
        // _ _ _
        // _ _ _
        // _ _ _
        let mut state = TicTacToeState::initial_state();

        // X _ _
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(0, 2)));

        // X _ O
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(2, 2)));

        // X _ O
        // _ _ _
        // X _ _
        state.apply_move(TicTacToeAction(BP::new(0, 0)));

        // White MUST block, or it will lose
        let mcts_results = mcts::<RcNode<_>, _, _>(
            state,
            PlayerColor::White,
            &mut util::get_rng_deterministic(),
        );

        let max_by_plays = mcts_results
            .iter()
            .max_by_key(|c| score_mcts_results_plays::<RcNode<_>, _>(c, PlayerColor::White))
            .unwrap();

        let expected_action = TicTacToeAction(BP::new(0, 1));
        assert_eq!(expected_action, max_by_plays.action);
    }

    #[test]
    pub fn mcts_score_results_plays_expects_always_picks_winning_move() {
        type BP = BoardPosition;
        // _ _ _
        // _ _ _
        // _ _ _
        let mut state = TicTacToeState::initial_state();

        // X _ _
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(0, 2)));

        // X _ O
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(2, 2)));

        // X _ O
        // _ _ _
        // X _ _
        state.apply_move(TicTacToeAction(BP::new(0, 0)));

        // X _ O
        // _ _ _
        // X _ O
        state.apply_move(TicTacToeAction(BP::new(2, 0)));

        // X _ O
        // _ X _
        // X _ O
        state.apply_move(TicTacToeAction(BP::new(1, 1)));

        // White MUST block, or it will lose
        let mcts_results = mcts::<RcNode<_>, _, _>(
            state,
            PlayerColor::White,
            &mut util::get_rng_deterministic(),
        );

        let max_by_plays = mcts_results
            .iter()
            .max_by_key(|c| score_mcts_results_plays::<RcNode<_>, _>(c, PlayerColor::White))
            .unwrap();

        let expected_action = TicTacToeAction(BP::new(2, 1));
        assert_eq!(expected_action, max_by_plays.action);
    }

    #[test]
    pub fn mcts_score_results_ratio_expects_always_picks_winning_move() {
        type BP = BoardPosition;
        // _ _ _
        // _ _ _
        // _ _ _
        let mut state = TicTacToeState::initial_state();

        // X _ _
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(0, 2)));

        // X _ O
        // _ _ _
        // _ _ _
        state.apply_move(TicTacToeAction(BP::new(2, 2)));

        // X _ O
        // _ _ _
        // X _ _
        state.apply_move(TicTacToeAction(BP::new(0, 0)));

        // X _ O
        // _ _ _
        // X _ O
        state.apply_move(TicTacToeAction(BP::new(2, 0)));

        // X _ O
        // _ X _
        // X _ O
        state.apply_move(TicTacToeAction(BP::new(1, 1)));

        // White MUST block, or it will lose
        let mcts_results = mcts::<RcNode<_>, _, _>(
            state,
            PlayerColor::White,
            &mut util::get_rng_deterministic(),
        );

        let max_by_plays = mcts_results
            .iter()
            .max_by_key(|c| score_mcts_results_ratio::<RcNode<_>, _>(c, PlayerColor::White))
            .unwrap();

        let expected_action = TicTacToeAction(BP::new(2, 1));
        assert_eq!(expected_action, max_by_plays.action);
    }
}
