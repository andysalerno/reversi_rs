use lib_boardgame::game_primitives::Game;
use lib_reversi::agents::human::HumanAgent;
use lib_reversi::agents::random_agent::RandomAgent;
use lib_reversi::reversi::Reversi;

fn main() {
    let white = HumanAgent;
    let black = RandomAgent;
    let b2 = HumanAgent;

    let mut game = if true {
        Reversi::new(white, black)
    } else {
        Reversi::new(b2, black)
    };
    //let mut game = Reversi::new(white, black);

    let game_result = game.play_to_end();
    println!("Result: {:?}", game_result);
}
