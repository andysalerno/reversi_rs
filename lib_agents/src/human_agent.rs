use lib_boardgame::game_primitives::{GameAgent, GameState};
use std::str::FromStr;

pub struct HumanAgent<TState: GameState>;

impl<TState: GameState> HumanAgent<TState> {
    fn get_user_move() -> TState::Move {
        use std::io::stdin;

        let position = loop {
            println!("Enter move x,y: ");

            let mut input = String::new();

            stdin()
                .read_line(&mut input)
                .expect("Couldn't capture user input.");

            let game_move = TState::Move::from_str(&input).unwrap();

            let nums: Vec<_> = input.split(',').map(|x| x.trim()).collect();

            if nums.len() != 2 {
                println!("Invalid input: {} -- expected format: col,row", input);
                continue;
            }

            let col = nums[0].parse::<usize>();
            let row = nums[1].parse::<usize>();

            if let (Ok(col), Ok(row)) = (col, row) {
                let board_pos = BoardPosition::new(col, row);
                if col > ReversiState::BOARD_SIZE || row >= ReversiState::BOARD_SIZE {
                    println!(
                        "Position out of bounds of board. Input: {:?}, actual board size: {}",
                        board_pos,
                        ReversiState::BOARD_SIZE
                    );
                } else {
                    break board_pos;
                }
            } else {
                println!("Didn't recognize input as a board position: {}", input);
            }
        };

        ReversiAction::Move {
            piece: color,
            position,
        }
    }
}

impl<TState: GameState> GameAgent<TState> for HumanAgent {
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        self.random_choice(&legal_moves)
    }
}

// impl GameAgent<ReversiState> for HumanAgent {
//     fn pick_move(&self, _state: &ReversiState, legal_moves: &[Action]) -> Action {
//         let color = match legal_moves[0] {
//             ReversiAction::PassTurn => {
//                 return legal_moves[0];
//             }
//             ReversiAction::Move {
//                 piece: piece_color, ..
//             } => piece_color,
//         };

//         let mut user_selected_move = Self::get_user_move(color);

//         while !legal_moves.contains(&user_selected_move) {
//             println!("Entered move is not legal.");
//             user_selected_move = Self::get_user_move(color);
//         }

//         user_selected_move
//     }
// }
