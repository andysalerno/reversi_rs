pub mod tic_tac_toe;
pub mod tic_tac_toe_gamestate;

use lib_boardgame::PlayerColor;

/// The size of the board.  E.x. if 3, the board is a 3x3 grid.
pub const BOARD_SIZE: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TicTacToePiece {
    X,
    O,
} 

impl TicTacToePiece {
    pub fn player_color(self) -> PlayerColor {
        match self {
            TicTacToePiece::X => PlayerColor::Black,
            TicTacToePiece::O => PlayerColor::White,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::tic_tac_toe_gamestate::{TicTacToeState, TicTacToeAction, BoardPosition};
    use lib_boardgame::{GameState, PlayerColor};
    use std::str::FromStr;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn winning_player_has_higher_score() {
        let mut state = TicTacToeState::initial_state();

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

        let white_score = state.player_score(PlayerColor::White);
        let black_score = state.player_score(PlayerColor::Black);

        assert_eq!(white_score, black_score, "No winner yet, so scores should be equal.");

        // Create this state:
        // XXX
        // _O_
        // __O
        state.apply_move(TicTacToeAction(BoardPosition::new(1, 2)));

        let white_score = state.player_score(PlayerColor::White);
        let black_score = state.player_score(PlayerColor::Black);

        assert!(black_score > white_score, "Black has won, so it should have the higher score.");
    }

    #[test]
    #[should_panic]
    fn applying_move_nonempty_location_expects_panic() {
        let mut state = TicTacToeState::initial_state();

        state.apply_move(TicTacToeAction::from_str("1,1").unwrap());

        // Another location is fine.
        state.apply_move(TicTacToeAction::from_str("2,1").unwrap());

        // But the same location should panic.
        state.apply_move(TicTacToeAction::from_str("1,1").unwrap());
    }
}
