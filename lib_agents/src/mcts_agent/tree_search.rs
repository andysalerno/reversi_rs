use crate::mcts_agent::mcts_data::{Data, MctsData, MctsResult};
use crate::util;

use lib_boardgame::GameResult;
use lib_boardgame::{GameState, PlayerColor};
use monte_carlo_tree::Node;
use std::borrow::Borrow;

pub(super) const TOTAL_SIMS: usize = 1000;

fn expand<TNode, TState>(node: &TNode) -> Option<TNode::ChildrenIter>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    node.data().mark_expanded();

    // todo: unnecessary optimization here?
    {
        let children = node.children().into_iter().collect::<Vec<_>>();
        assert!(children.is_empty());
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
fn backprop_saturation<TNode, TState>(node: &TNode)
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut parent_node = node.parent();

    while let Some(p) = parent_node {
        let parent = p.borrow();
        let data = parent.data();
        data.increment_saturated_children_count();

        if !data.is_saturated() {
            // we incremented but we're still not saturated,
            // so don't keep going.
            return;
        }

        parent_node = parent.parent();
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
    let data = node.data();
    data.increment_plays();

    let incr_wins = (result == GameResult::BlackWins && color == PlayerColor::Black)
        || (result == GameResult::WhiteWins && color == PlayerColor::White);

    if incr_wins {
        data.increment_wins();
    }

    let mut parent_node = node.parent();

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

/// Repeatedly select nodes down the tree until a leaf is reached.
/// If the given root node is already a leaf,
/// or is saturated, it is returned.
#[allow(unused)]
fn select_to_leaf<TNode, TState>(root: &TNode, player_color: PlayerColor) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child = select_child::<TNode, TState>(cur_node.clone());

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

/// This proved to give better results (marginally) than the non-rand version.
fn select_to_leaf_rand<TNode, TState, Rng>(root: &TNode, player_color: PlayerColor, rng: &mut Rng) -> TNode::Handle
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    Rng: rand::Rng,
{
    let mut cur_node = root.get_handle();

    loop {
        let selected_child =
            if player_color == cur_node.borrow().data().state().current_player_turn() {
                let selected_child: Option<TNode::Handle> =
                    select_child::<TNode, TState>(cur_node.clone());
                selected_child
            } else {
                let all_children = cur_node
                    .borrow()
                    .children()
                    .into_iter()
                    .filter(|c| !c.borrow().data().is_saturated())
                    .collect::<Vec<_>>();

                let selected_child = util::random_pick(&all_children, rng);
                selected_child.cloned()
            };

        if selected_child.is_none() {
            return cur_node;
        }

        cur_node = selected_child.unwrap();
    }
}

/// For all children of the given node, assign each one a score,
/// and return the child with the highest score (ties broken by the first)
/// or None if there are no children (or if every child is already saturated).
fn select_child<TNode, TState>(root: TNode::Handle) -> Option<TNode::Handle>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let child_nodes = root.borrow().children();

    child_nodes
        .into_iter()
        .filter(|n| !n.borrow().data().is_saturated())
        .max_by(|a, b| {
            score_node(a.borrow())
                .partial_cmp(&score_node(b.borrow()))
                .unwrap()
        })
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
    let bias = 2_f32;

    (wins / plays) + (bias * f32::sqrt(f32::ln(parent_plays) / plays))
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
        (game_result == GameResult::BlackWins && color == PlayerColor::Black)
            || (game_result == GameResult::WhiteWins && color == PlayerColor::White)
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

pub fn mcts<TNode, TState, Rng>(state: TState, player_color: PlayerColor, rng: &mut Rng) -> Vec<MctsResult<TState>>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
    Rng: rand::Rng,
{
    let turn_root = TNode::new_root(MctsData::new(&state, 0, 0, None));

    for _ in 0..TOTAL_SIMS {
        // if our previous work has saturated the tree, we can break early,
        // since we have visited every single outcome already

        if turn_root.borrow().data().is_saturated() {
            break;
        }

        // select the leaf node that we will expand
        let leaf = select_to_leaf_rand(turn_root.borrow(), player_color, rng);

        let leaf = leaf.borrow();

        // expand (create the child nodes for the selected leaf node)
        let expanded_children = expand(leaf);

        if expanded_children.is_none() {
            // we've reached a terminating node in the game
            let sim_result = simulate(leaf, rng);
            backprop_sim_result(leaf, sim_result, player_color);

            leaf.data().set_end_state_result(sim_result);

            // we now know we have selected a terminating node (which is saturated by definition),
            // so we can update the parent's saturated child count.
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

    let turn_root = turn_root.borrow();
    let state_children = turn_root.children();

    let results: Vec<MctsResult<TState>> = state_children
        .into_iter()
        .map(|c| c.borrow().data().into())
        .collect::<Vec<_>>();

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::TicTacToeState;
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
    fn new_child_works() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());
        let child = tree_root.new_child(data.clone());

        assert_eq!(1, tree_root.children().into_iter().count());
        assert!(child.borrow().parent().is_some());
        assert!(tree_root.parent().is_none());
    }

    #[test]
    fn backprop_works_one_node_black() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::BlackWins, PlayerColor::Black);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(1, tree_root.data().wins());
    }

    #[test]
    fn backprop_works_one_node_white() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::WhiteWins, PlayerColor::White);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(1, tree_root.data().wins());
    }

    #[test]
    fn backprop_works_one_node_loss() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::BlackWins, PlayerColor::White);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, tree_root.data().wins());
    }

    #[test]
    fn backprop_works_one_node_tie() {
        let data = make_test_data();
        let tree_root = make_node(data.clone());

        backprop_sim_result(&tree_root, GameResult::Tie, PlayerColor::White);

        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, tree_root.data().wins());
    }

    #[test]
    fn expand_works() {
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
    fn backprop_works_several_nodes() {
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
        ); // TODO

        assert_eq!(1, child_level_3.borrow().data().plays());
        assert_eq!(1, child_level_2.borrow().data().plays());
        assert_eq!(1, child_level_1.borrow().data().plays());
        assert_eq!(1, tree_root.data().plays());
        assert_eq!(0, child_level_4.borrow().data().plays());
    }

    // TODO: need to figure out how to make the compiler happy on this one...
    // #[test]
    // fn select_child_works() {
    //     let data = make_test_data();

    //     let tree_root = make_node(data.clone());
    //     let child_level_1 = tree_root.new_child(data.clone());
    //     let child_level_2 = child_level_1.borrow().new_child(data.clone());
    //     let child_level_3 = child_level_2.borrow().new_child(data.clone());
    //     let child_level_4 = child_level_3.borrow().new_child(data.clone());
    //     let child_level_4b = child_level_3.borrow().new_child(data.clone());
    //     // let data = MctsData::new(&TicTacToeState::new(), 0, 0, None);

    //     // let tree_root = RcNode::new_root(data.clone());
    //     // let child_level_1: RcNode<MctsData<TicTacToeState>>  = tree_root.new_child(data.clone());
    //     // let child_level_2 = child_level_1.borrow(); //.new_child(data.clone());
    //     // let child_level_3 = child_level_2.borrow().new_child(data.clone());
    //     // let child_level_4 = child_level_3.borrow().new_child(data.clone());
    //     // let child_level_4b = child_level_3.borrow().new_child(data.clone());

    //     child_level_1.borrow().data().set_children_count(1);
    //     child_level_2.borrow().data().set_children_count(1);
    //     child_level_3.borrow().data().set_children_count(2);
    //     child_level_4.borrow().data().set_children_count(1);
    //     child_level_4b.borrow().data().set_children_count(1);

    //     backprop_sim_result(child_level_3.borrow(), GameResult::BlackWins, PlayerColor::Black); // TODO: all
    //     backprop_sim_result(child_level_4.borrow(), GameResult::BlackWins, PlayerColor::Black);
    //     backprop_sim_result(child_level_4.borrow(), GameResult::BlackWins, PlayerColor::Black);
    //     backprop_sim_result(child_level_4.borrow(), GameResult::BlackWins, PlayerColor::Black);
    //     backprop_sim_result(child_level_4b.borrow(), GameResult::BlackWins, PlayerColor::Black);

    //     assert!(!child_level_3.borrow().data().is_saturated());

    //     let selected =
    //         select_child(child_level_3);
    //         // .expect("the child should have been selected.");

    //     // assert_eq!(1, selected.borrow().data().plays());
    // }

    #[test]
    fn select_to_leaf_works() {
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
        ); // TODO: all
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

        let leaf = select_to_leaf(&tree_root, PlayerColor::Black);

        let leaf = leaf.borrow();

        assert_eq!(2, leaf.data().plays());
    }

    #[test]
    fn select_to_leaf_when_already_leaf_returns_self() {
        let data = MctsData::new(&make_test_state(), 10, 10, None);

        let tree_root = RcNode::new_root(data.clone());

        let leaf = select_to_leaf(&tree_root, PlayerColor::Black);

        assert_eq!(10, leaf.data().plays());
        assert_eq!(10, leaf.data().wins());
    }

    #[test]
    fn backprop_saturation_becomes_saturated() {
        let data = {
            let mut state = make_test_state();
            state.initialize_board();
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

        for child in children.iter().skip(1) {
            backprop_saturation(child.borrow());
        }

        assert!(
            !tree_root.data().is_saturated(),
            "Every child is saturated, but not every child has had its saturation status backpropagated, so the root should not be considered saturated."
        );

        // backprop the one remaining child.
        backprop_saturation(children[0].borrow());

        assert!(
            tree_root.data().is_saturated(),
            "Now that every child has had its saturation backpropagated, the parent should be considered saturated as well."
        );
    }

    #[test]
    fn backprop_multi_levels_works() {
        let data = {
            let mut state = make_test_state();
            state.initialize_board();
            MctsData::new(&state, 0, 0, None)
        };

        let tree_root = make_node(data.clone());

        let children_1 = expand(tree_root.borrow())
            .expect("must have children")
            .into_iter()
            .collect::<Vec<_>>();

        let child_a = &children_1[0];
        let child_b = &children_1[1];

        let grandchildren_a = expand(child_a.borrow())
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        let grandchildren_b = expand(child_b.borrow())
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        assert!(!tree_root.data().is_saturated());
        assert!(!child_a.borrow().data().is_saturated());
        assert!(!child_b.borrow().data().is_saturated());

        for grandchild in grandchildren_a {
            backprop_saturation(grandchild.borrow());
        }

        assert!(!tree_root.data().is_saturated());
        assert!(!child_b.borrow().data().is_saturated());

        assert!(child_a.borrow().data().is_saturated());

        for grandchild in grandchildren_b {
            backprop_saturation(grandchild.borrow());
        }

        assert!(!tree_root.data().is_saturated());
        assert!(child_a.borrow().data().is_saturated());
        assert!(child_b.borrow().data().is_saturated());

        for child in children_1.iter().skip(2) {
            backprop_saturation(child.borrow());
        }

        assert!(tree_root.data().is_saturated());
        assert!(child_a.borrow().data().is_saturated());
        assert!(child_b.borrow().data().is_saturated());
    }

    #[test]
    fn score_node_works() {
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        // all children of the same parent
        let child_a = tree_root.borrow().new_child(data.clone());
        let child_b = tree_root.borrow().new_child(data.clone());
        let child_c = tree_root.borrow().new_child(data.clone());
        let child_d = tree_root.borrow().new_child(data.clone());

        // "visit" each child a different amount of times
        // child a: three visits
        let player_agent_color = PlayerColor::White;
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color); // TODO: all
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color);
        backprop_sim_result(child_a.borrow(), GameResult::BlackWins, player_agent_color);

        // child b: two visits
        backprop_sim_result(child_b.borrow(), GameResult::BlackWins, player_agent_color);
        backprop_sim_result(child_b.borrow(), GameResult::BlackWins, player_agent_color);

        // child c: one visit
        backprop_sim_result(child_c.borrow(), GameResult::BlackWins, player_agent_color);

        assert_eq!(1.5456431, score_node(child_a.borrow()));
        assert_eq!(1.8930185, score_node(child_b.borrow()));
        assert_eq!(2.6771324, score_node(child_c.borrow()));
        assert_eq!(
            340282350000000000000000000000000000000f32,
            score_node(child_d.borrow())
        );
    }

    #[test]
    fn simulate_runs_to_completion_and_terminates() {
        let mut initial_state = make_test_state();
        initial_state.initialize_board();
        let data = make_test_data();

        let tree_root = make_node(data.clone());

        // let _sim_result = simulate(&tree_root) TODO ;
    }

}
