mod agents;
mod game_primitives;
mod reversi;

#[cfg(test)]
mod tests {
    use crate::agents::random_agent::RandomAgent;
    use crate::game_primitives::PlayerColor;
    use crate::reversi::Reversi;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let white = RandomAgent::new(PlayerColor::White);
        let black = RandomAgent::new(PlayerColor::Black);

        let game = Reversi::new(white, black);
    }
}
