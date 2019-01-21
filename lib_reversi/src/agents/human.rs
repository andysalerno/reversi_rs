use crate::reversi_gamestate::ReversiState;
use crate::{BoardPosition, ReversiAction, ReversiPiece};
use lib_boardgame::game_primitives::{GameAgent, GameState};

pub struct HumanAgent;

type Action = <ReversiState as GameState>::Move;

impl HumanAgent {
    fn get_user_move(color: ReversiPiece) -> Action {
        use std::io::stdin;

        println!("Enter move x,y: ");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("Couldn't capture user input.");

        let nums: Vec<_> = dbg!(input.split(',').map(|x| x.trim()).collect());

        let col = nums[0]
            .parse::<usize>()
            .expect(&format!("illegal col: {}", nums[0]));
        let row = nums[1]
            .parse::<usize>()
            .expect(&format!("illegal row: {}", nums[1]));

        let position = BoardPosition::new(col, row);

        ReversiAction::Move {
            piece: color,
            position: position,
        }
    }
}

impl GameAgent<ReversiState> for HumanAgent {
    fn pick_move(&self, _state: &ReversiState, legal_moves: &[Action]) -> Action {
        let color = match legal_moves[0] {
            ReversiAction::PassTurn => {
                return legal_moves[0];
            }
            ReversiAction::Move {
                piece: piece_color,
                position: _,
            } => piece_color,
        };

        let mut user_selected_move = Self::get_user_move(color);

        while !legal_moves.contains(&user_selected_move) {
            println!("Entered move is not legal.");
            user_selected_move = Self::get_user_move(color);
        }

        user_selected_move
    }
}
