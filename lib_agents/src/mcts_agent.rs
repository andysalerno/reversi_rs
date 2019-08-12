pub mod tree_search;

use crate::util::get_rng;
use lib_boardgame::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::{
    monte_carlo_data::MctsData, monte_carlo_data::MctsResult, rc_tree::RcNode, tree::Node,
};
use std::marker::PhantomData;

pub struct MctsAgent<TState, TNode = RcNode<MctsData<TState>>>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    color: PlayerColor,

    _phantom_a: PhantomData<TState>,
    _phantom_b: PhantomData<TNode>,
}

impl<TState, TNode> MctsAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
{
    pub fn new(color: PlayerColor) -> Self {
        MctsAgent {
            color,
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        }
    }
}

impl<TState, TNode> GameAgent<TState> for MctsAgent<TState, TNode>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState + Sync,
{
    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let result = match self.color {
            PlayerColor::Black => perform_mcts_single_threaded::<TNode, TState>(state, self.color),
            PlayerColor::White => perform_mcts_multithreaded::<TNode, TState>(state, self.color),
        };

        let white_wins = if self.color == PlayerColor::White {
            result.wins
        } else {
            result.plays - result.wins
        };

        println!("{}", pretty_ratio_bar_text(20, white_wins, result.plays));

        result.action
    }
}

fn pretty_ratio_bar_text(
    len_chars: usize,
    numerator_white_wins: usize,
    denominator_plays: usize,
) -> String {
    let mut text_bar = String::with_capacity(len_chars + 7);

    text_bar.push_str("B [");

    let bar_len = (numerator_white_wins * len_chars) / denominator_plays;
    let bar_txt = "=".repeat(bar_len);
    text_bar.push_str(&bar_txt);
    text_bar.push('|');

    let bar_empty = " ".repeat(len_chars - bar_len);
    text_bar.push_str(&bar_empty);

    text_bar.push_str("] W");

    text_bar
}

fn perform_mcts_single_threaded<TNode, TState>(
    state: &TState,
    color: PlayerColor,
) -> MctsResult<TState>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let mut results =
        tree_search::mcts_result::<TNode, TState, _>(state.clone(), color, &mut get_rng());

    for r in &results {
        let sat_display = if r.is_saturated { "(S)" } else { "" };

        println!(
            "Action: {:?} Plays: {} Wins: {} ({:.2}) {}",
            r.action,
            r.plays,
            r.wins,
            r.wins as f32 / r.plays as f32,
            sat_display,
        );
    }

    results.pop().expect("Must be at least one result")
}

fn perform_mcts_multithreaded<TNode, TState>(
    state: &TState,
    player_color: PlayerColor,
) -> MctsResult<TState>
where
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState + Sync,
{
    let color = player_color;

    let mut results = {
        let mut result_1 = None;
        let mut result_2 = None;
        let mut result_3 = None;
        let mut result_4 = None;

        let state_1 = state.clone();
        let state_2 = state.clone();
        let state_3 = state.clone();
        let state_4 = state.clone();

        rayon::scope(|s| {
            s.spawn(|_| {
                result_1 = Some(tree_search::mcts_result::<TNode, TState, _>(
                    state_1,
                    color,
                    &mut get_rng(),
                ))
            });
            s.spawn(|_| {
                result_2 = Some(tree_search::mcts_result::<TNode, TState, _>(
                    state_2,
                    color,
                    &mut get_rng(),
                ))
            });
            s.spawn(|_| {
                result_3 = Some(tree_search::mcts_result::<TNode, TState, _>(
                    state_3,
                    color,
                    &mut get_rng(),
                ))
            });
            s.spawn(|_| {
                result_4 = Some(tree_search::mcts_result::<TNode, TState, _>(
                    state_4,
                    color,
                    &mut get_rng(),
                ))
            });
        });

        let mut result_1 = result_1.unwrap();

        let actions_count = result_1.len();

        let subsequent_results = vec![result_2, result_3, result_4];

        // aggregate all the action play/win values into result_1
        for i in 0..actions_count {
            let result_1_action = result_1.get_mut(i).unwrap();

            for subsequent_result in subsequent_results.iter().filter(|r| r.is_some()) {
                let matching_action = subsequent_result
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|r| r.action == result_1_action.action)
                    .unwrap();

                result_1_action.plays += matching_action.plays;
                result_1_action.wins += matching_action.wins;
            }
        }

        result_1
    };

    for r in &results {
        let sat_display = if r.is_saturated { "(S)" } else { "" };

        println!(
            "Action: {:?} Plays: {} Wins: {} ({:.2}) {}",
            r.action,
            r.plays,
            r.wins,
            r.wins as f32 / r.plays as f32,
            sat_display,
        );
    }

    results.pop().expect("Must have been at least one action.")
}

#[cfg(test)]
mod tests {
    use super::*;

    use lib_boardgame::{Game, GameState};
    use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::{BoardPosition, TicTacToeAction};

    #[test]
    fn tree_search_always_picks_winning_move() {
        let black_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let white_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::White);

        let mut game = TicTacToe::new(white_agent, black_agent);

        let state = game.game_state_mut();

        // Start with black's turn
        assert_eq!(state.current_player_turn(), PlayerColor::Black);

        // Create this state:
        // X__
        // ___
        // ___
        state.apply_move(TicTacToeAction(BoardPosition::new(0, 2)));

        assert_eq!(state.current_player_turn(), PlayerColor::White);

        // Create this state:
        // X__
        // ___
        // __O
        state.apply_move(TicTacToeAction(BoardPosition::new(2, 0)));

        assert_eq!(state.current_player_turn(), PlayerColor::Black);

        // Create this state:
        // X_X
        // ___
        // __O
        state.apply_move(TicTacToeAction(BoardPosition::new(2, 2)));

        assert_eq!(state.current_player_turn(), PlayerColor::White);

        // Create this state:
        // X_X
        // _O_
        // __O
        state.apply_move(TicTacToeAction(BoardPosition::new(1, 1)));

        assert_eq!(state.current_player_turn(), PlayerColor::Black);
        let legal_moves = state.legal_moves(PlayerColor::Black);

        let test_black_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let mcts_chosen_move = test_black_agent.pick_move(state, &legal_moves);

        // The agent MUST pick the winning move:
        //  V
        // XXX
        // _O_
        // __O
        assert_eq!(TicTacToeAction(BoardPosition::new(1, 2)), mcts_chosen_move);
    }
}
