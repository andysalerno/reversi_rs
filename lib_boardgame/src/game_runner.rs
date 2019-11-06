use crate::{GameAction, GameAgent, GameResult, GameState, PlayerColor};
use lib_printer::{out, out_impl};

/// A trait that describes a game runner.
pub trait GameRunner<T: GameState> {
    fn play_to_end(black_agent: &dyn GameAgent<T>, white_agent: &dyn GameAgent<T>) -> GameResult;
}

/// A trivial, general-purpose implementation of a GameRunner.
/// Probably all you need to run most games.
pub struct GeneralGameRunner;

fn player_take_turn<S>(game_state: &mut S, agent: &dyn GameAgent<S>) -> S::Action
where
    S: GameState,
{
    let player_color = agent.player_color();
    let legal_moves = game_state.legal_moves(player_color);

    let selected_action = agent.pick_move(game_state, legal_moves);

    if legal_moves
        .iter()
        .find(|&&m| m == selected_action)
        .is_none()
    {
        panic!("Agent provided a move that is illegal.");
    }

    if legal_moves.len() == 1 && legal_moves[0].is_forced_pass() {
        out!(
            "Player {:?} has no options, so they pass their turn.",
            player_color
        );
    }

    selected_action
}

impl<T> GameRunner<T> for GeneralGameRunner
where
    T: GameState,
{
    fn play_to_end(black_agent: &dyn GameAgent<T>, white_agent: &dyn GameAgent<T>) -> GameResult {
        let mut game_state = T::initial_state();

        while !game_state.is_game_over() {
            out!("{}", game_state.human_friendly());
            let cur_player_color = game_state.current_player_turn();

            let agent_to_play = match cur_player_color {
                PlayerColor::Black => black_agent,
                PlayerColor::White => white_agent,
            };

            let selected_action = player_take_turn(&mut game_state, agent_to_play);

            out!(
                "Player {:?} picked move {:?}",
                cur_player_color,
                selected_action
            );

            game_state.apply_move(selected_action);

            black_agent.observe_action(cur_player_color, selected_action, &game_state);
            white_agent.observe_action(cur_player_color, selected_action, &game_state);
        }

        out!("{}", game_state.human_friendly());

        game_state
            .game_result()
            .expect("The game is over, so there must be a game result.")
    }
}
