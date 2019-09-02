pub mod tree_search;
pub mod tree_search_par;

use crate::util::get_rng;
use crossbeam::thread;
use lib_boardgame::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::{
    amonte_carlo_data::AMctsData, arc_tree::ArcNode, monte_carlo_data::MctsResult, tree::Node,
};
use std::marker::PhantomData;
use std::marker::Sync;
use std::sync::Mutex;
use std::time::Instant;

pub struct MctsAgent<TState, TNode = ArcNode<AMctsData<TState>>>
where
    TState: GameState,
    TNode: Node<Data = AMctsData<TState>>,
{
    color: PlayerColor,

    _phantom_a: PhantomData<TState>,
    _phantom_b: PhantomData<TNode>,
}

impl<TState, TNode> MctsAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = AMctsData<TState>>,
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
    TNode: Node<Data = AMctsData<TState>> + Sync,
    TState: GameState + Sync,
{
    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let result = match self.color {
            PlayerColor::Black => perform_mcts_par::<TNode, TState>(state, self.color, 1),
            PlayerColor::White => perform_mcts_par::<TNode, TState>(state, self.color, 1),
            // PlayerColor::White => perform_mcts_single_threaded::<TNode, TState>(state, self.color),
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

fn perform_mcts_par<TNode, TState>(
    state: &TState,
    player_color: PlayerColor,
    thread_count: usize,
) -> MctsResult<TState>
where
    TNode: Node<Data = AMctsData<TState>> + Sync,
    TState: GameState + Sync,
{
    let results = tree_search_par::mcts_result::<TNode, TState>(state.clone(), player_color);

    if results.iter().all(|r| r.is_saturated) {
        results
            .iter()
            .max_by_key(|r| (r.wins * 10000) / r.plays)
            .expect("Must have been a max result")
            .clone()
    } else {
        results
            .iter()
            .max_by_key(|r| r.plays)
            .expect("Must have been a max result")
            .clone()
    }
}

// fn perform_mcts_multithreaded<TNode, TState>(
//     state: &TState,
//     player_color: PlayerColor,
//     thread_count: usize,
// ) -> MctsResult<TState>
// where
//     TNode: Node<Data = MctsData<TState>>,
//     TState: GameState + Sync,
// {
//     let thread_results = Mutex::new(Vec::new());

//     let now = Instant::now();
//     thread::scope(|s| {
//         for _ in 0..thread_count {
//             s.spawn(|_| {
//                 let result = tree_search::mcts_result::<TNode, TState, _>(
//                     state.clone(),
//                     player_color,
//                     &mut get_rng(),
//                 );
//                 thread_results
//                     .lock()
//                     .expect("Could not lock results")
//                     .push(result);
//             });
//         }
//     })
//     .unwrap();
//     let elapsed = now.elapsed();

//     let mut all_thread_results = thread_results.into_inner().unwrap();

//     // Make a result that is the aggregation of the many results
//     let mut first_thread_result = all_thread_results.remove(0);

//     for mut each_action_result in first_thread_result.iter_mut() {
//         for other_threads_results in &all_thread_results {
//             let same_move_result = other_threads_results
//                 .iter()
//                 .find(|&r| r.action == each_action_result.action)
//                 .expect("All results must contain the same moves.");

//             each_action_result.plays += same_move_result.plays;
//             each_action_result.wins += same_move_result.wins;
//         }
//     }

//     let total_plays = first_thread_result.iter().map(|r| r.plays).sum::<usize>();
//     dbg!(total_plays);

//     let plays_per_sec = total_plays as f64 / (elapsed.as_millis() as f64 / 1_000_f64);
//     println!("Plays per sec: {:.0}", plays_per_sec);

//     for action_result in &first_thread_result {
//         let sat_display = if action_result.is_saturated {
//             "(S)"
//         } else {
//             ""
//         };

//         println!(
//             "Action: {:?} Plays: {} Wins: {} ({:.2}) {}",
//             action_result.action,
//             action_result.plays,
//             action_result.wins,
//             action_result.wins as f32 / action_result.plays as f32,
//             sat_display,
//         );
//     }

//     if first_thread_result.iter().all(|r| r.is_saturated) {
//         first_thread_result
//             .iter()
//             .max_by_key(|r| (r.wins * 10000) / r.plays)
//             .expect("Must have been a max result")
//             .clone()
//     } else {
//         first_thread_result
//             .iter()
//             .max_by_key(|r| r.plays)
//             .expect("Must have been a max result")
//             .clone()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    use lib_boardgame::{Game, GameState};
    use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::{BoardPosition, TicTacToeAction};

    #[test]
    fn tree_search_always_picks_winning_move() {
        let black_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let white_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::White);

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

        let test_black_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let mcts_chosen_move = test_black_agent.pick_move(state, &legal_moves);

        // The agent MUST pick the winning move:
        //  V
        // XXX
        // _O_
        // __O
        assert_eq!(TicTacToeAction(BoardPosition::new(1, 2)), mcts_chosen_move);
    }
}
