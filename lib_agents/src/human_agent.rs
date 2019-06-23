use lib_boardgame::{GameAgent, GameState, PlayerColor};
use std::marker::PhantomData;
use std::str::FromStr;

pub struct HumanAgent<TState: GameState>
where
    TState::Move: FromStr,
{
    _phantom: PhantomData<TState>,
    player_color: PlayerColor,
}

impl<TState: GameState> HumanAgent<TState>
where
    TState::Move: FromStr,
    <TState::Move as FromStr>::Err: std::fmt::Debug,
{
    pub fn new(player_color: PlayerColor) -> Self {
        Self {
            _phantom: Default::default(),
            player_color,
        }
    }

    fn get_user_move() -> TState::Move {
        use std::io::stdin;

        println!("Enter move x,y: ");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .expect("Couldn't capture user input.");

        TState::Move::from_str(&input).unwrap()
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
