use lib_boardgame::{GameAgent, GameState, PlayerColor, GameMoveFromStr};
use std::marker::PhantomData;
use std::str::FromStr;

pub struct HumanAgent<TState: GameState>
where
    TState::Move: FromStr,
{
    player_color: PlayerColor,
    _phantom: PhantomData<TState>,
}

impl<TState: GameState> HumanAgent<TState>
where
    TState::Move: GameMoveFromStr,
    <TState::Move as FromStr>::Err: std::fmt::Debug,
{
    pub fn new(player_color: PlayerColor) -> Self {
        Self {
            _phantom: Default::default(),
            player_color
        }
    }

    fn prompt_input(&self) -> TState::Move {
        use std::io::stdin;

        println!("Enter move x,y: ");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .expect("Couldn't capture user input.");

        let result = GameMoveFromStr::from_str(&input, self.player_color);

        match result {
            Ok(r) => return r,
            _ => {
                println!("Invalid input.  Try again.");
                return self.prompt_input();
            }
        }
    }
}

impl<TState: GameState> GameAgent<TState> for HumanAgent<TState>
where
    TState::Move: GameMoveFromStr,
    <TState::Move as FromStr>::Err: std::fmt::Debug,
{
    fn pick_move(&self, _state: &TState, legal_moves: &[TState::Move]) -> TState::Move {
        loop {
            let user_input = self.prompt_input();

            if legal_moves.iter().find(|&&m| m == user_input).is_none() {
                println!("The provided move was not valid. Try again.");
            }
            else {
                return user_input;
            }
        }
    }
}
