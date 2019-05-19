use lib_boardgame::{GameAgent, GameState};
use std::marker::PhantomData;
use std::str::FromStr;

pub struct HumanAgent<TState: GameState>
where
    TState::Move: FromStr,
{
    _phantom: PhantomData<TState>,
}

impl<TState: GameState> HumanAgent<TState>
where
    TState::Move: FromStr,
    <TState::Move as FromStr>::Err: std::fmt::Debug,
{
    fn get_user_move() -> TState::Move {
        use std::io::stdin;

        println!("Enter move x,y: ");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .expect("Couldn't capture user input.");

        let game_move = TState::Move::from_str(&input).unwrap();

        game_move
    }
}

impl<TState: GameState> GameAgent<TState> for HumanAgent<TState>
where
    TState::Move: FromStr,
    <TState::Move as FromStr>::Err: std::fmt::Debug,
{
    fn pick_move(&self, _state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        Self::get_user_move()
    }
}
