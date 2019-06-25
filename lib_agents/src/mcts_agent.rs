mod tree_search;
mod mcts_data;

use lib_boardgame::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::rc_tree::RcNode;
use monte_carlo_tree::Node;
use std::marker::PhantomData;
use std::time::Instant;
use mcts_data::{Data, MctsData};
use rayon::prelude::*;

const TOTAL_SIMS: u128 = 1000;

pub struct MctsAgent<TState, TNode = RcNode<MctsData<TState>>>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
    // <TState as lib_boardgame::GameState>::Move: std::marker::Send
{
    color: PlayerColor,

    // todo: the fact that I require these lines must mean something is wrong...
    _phantom_a: PhantomData<TState>,
    _phantom_b: PhantomData<TNode>,
}

impl<TState, TNode> MctsAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = MctsData<TState>>,
    // <TState as lib_boardgame::GameState>::Move: std::marker::Send
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
    TNode: Node<Data = MctsData<TState>>,
    TState: GameState + Sync,
{
    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let now = Instant::now();

        let state_a = state.clone();
        let state_b = state.clone();
        let color = self.color;

        let (mcts_result, _) = rayon::join(
            || tree_search::mcts::<TNode, TState>(state_a, color),
            || tree_search::mcts::<TNode, TState>(state_b, color),
        );

        let elapsed_micros = now.elapsed().as_micros();
        println!(
            "{} sims total. {:.2} sims/sec.",
            TOTAL_SIMS,
            (TOTAL_SIMS as f64 / elapsed_micros as f64) * 1_000_000f64
        );

        let max_scoring_result = mcts_result
            .into_iter()
            .max_by_key(|c| tree_search::score_mcts_results::<TNode, TState>(c, self.color))
            .unwrap();

        println!(
            "Plays: {} Wins: {} ({:.2})",
            max_scoring_result.plays,
            max_scoring_result.wins,
            max_scoring_result.wins as f32 / max_scoring_result.plays as f32,
        );

        max_scoring_result.action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lib_boardgame::{Game, GameMove, GameState};
    use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::{BoardPosition, TicTacToeAction, TicTacToeState};
    use std::borrow::Borrow;
    fn make_test_state() -> impl GameState {
        TicTacToeState::new()
    }

    fn make_node<G: GameState>(data: MctsData<G>) -> impl Node<Data = MctsData<G>> {
        RcNode::new_root(data)
    }

    fn make_test_data() -> MctsData<impl GameState> {
        MctsData::new(&make_test_state(), 0, 0, None)
    }

    #[test]
    fn tree_search_always_picks_winning_move() {
        let black_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let white_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::White);

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

        let test_black_agent: MctsAgent<_, RcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let mcts_chosen_move = test_black_agent.pick_move(state, &legal_moves);

        // The agent MUST pick the winning move:
        //  V
        // XXX
        // _O_
        // __O
        assert_eq!(TicTacToeAction(BoardPosition::new(1, 2)), mcts_chosen_move);
    }
}
