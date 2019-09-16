use super::tree_search_par;
use lib_boardgame::{GameAgent, GameState, PlayerColor};
use monte_carlo_tree::{
    amonte_carlo_data::AMctsData, arc_tree::ArcNode, atree::ANode, monte_carlo_data::MctsResult,
    tree::Node,
};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::Sync;
use std::time::Instant;
use std::ops::Deref;

pub struct MctsAgent<TState, TNode = ArcNode<AMctsData<TState>>>
where
    TState: GameState,
    TNode: Node<Data = AMctsData<TState>>,
{
    color: PlayerColor,
    current_state_root: RefCell<Option<TNode::Handle>>,
}

impl<TState, TNode> MctsAgent<TState, TNode>
where
    TState: GameState,
    TNode: Node<Data = AMctsData<TState>>,
{
    pub fn new(color: PlayerColor) -> Self {
        MctsAgent {
            color,
            current_state_root: RefCell::new(None),
        }
    }

    fn walk_tree_to_child(&self, action: TState::Move) {
        // IDEA: half threads are "win seekers" and other half is "loss seeker"
        // (i.e. explores as though we're playing for the opponent)
        let root_handle = self
            .current_root_handle()
            .expect("Must have a root node to seek through.");
        let cur_state_node = root_handle.borrow();
        let children = cur_state_node.children_read().deref();

        let resulting_child = children
            .iter()
            .find(|&c| c.borrow().data().action().expect("action") == action)
            .unwrap_or_else(|| {
                panic!(
                    "The provided move {:?} was not in the set of available moves: {:?}",
                    action,
                    children
                        .iter()
                        .map(|c| c.borrow().data().action().unwrap())
                        .collect::<Vec<_>>()
                )
            })
            .clone();

        *self.current_state_root.borrow_mut() = Some(resulting_child);
    }

    fn current_root_handle(&self) -> Option<TNode::Handle> {
        let cur_state_node = self.current_state_root.borrow();
        let cur_state_node = cur_state_node.as_ref()?;

        Some(cur_state_node.clone())
    }

    fn reset_root_handle(&self, state: &TState) -> TNode::Handle {
        let fresh_data = AMctsData::new(state.clone(), 0, 0, None);
        let fresh_root = TNode::new_root(fresh_data);

        let mut opt = self.current_state_root.borrow_mut();

        *opt = Some(fresh_root.clone());

        fresh_root.clone()
    }
}

impl<TState, TNode> GameAgent<TState> for MctsAgent<TState, TNode>
where
    TNode: ANode<Data = AMctsData<TState>>,
    TState: GameState + Sync,
{
    fn observe_action(&self, _player: PlayerColor, action: TState::Move, _result: &TState) {
        if self.current_root_handle().is_some() {
            self.walk_tree_to_child(action);
        }
    }

    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let root_handle = self
            .current_root_handle()
            .unwrap_or_else(|| self.reset_root_handle(state));

        let result = match self.color {
            PlayerColor::Black => perform_mcts_par::<TNode, TState>(root_handle, self.color, 2),
            PlayerColor::White => perform_mcts_par::<TNode, TState>(root_handle, self.color, 2),
        };

        let white_wins = if self.color == PlayerColor::White {
            result.wins
        } else {
            result.plays - result.wins
        };

        println!("{}", pretty_ratio_bar_text(20, white_wins, result.plays));

        result.action
    }
}

fn pretty_ratio_bar_text(
    len_chars: usize,
    numerator_white_wins: usize,
    denominator_plays: usize,
) -> String {
    let mut text_bar = String::with_capacity(len_chars + 7);

    text_bar.push_str("B [");

    let bar_len = (numerator_white_wins * len_chars) / denominator_plays;
    let bar_txt = "=".repeat(bar_len);
    text_bar.push_str(&bar_txt);
    text_bar.push('|');

    let bar_empty = " ".repeat(len_chars - bar_len);
    text_bar.push_str(&bar_empty);

    text_bar.push_str("] W");

    text_bar
}

fn perform_mcts_par<TNode, TState>(
    root: TNode::Handle,
    player_color: PlayerColor,
    thread_count: usize,
) -> MctsResult<TState>
where
    TNode: ANode<Data = AMctsData<TState>> + Sync,
    TState: GameState + Sync,
{
    let total_plays_before = root
        .borrow()
        .children_read()
        .into_iter()
        .map(|c| c.borrow().data().plays())
        .sum::<usize>();

    let now = Instant::now();
    let results = tree_search_par::mcts_result::<TNode, TState>(root, player_color, thread_count);
    let elapsed = now.elapsed();

    // Some friendly UI output
    {
        let total_plays = results.iter().map(|r| r.plays).sum::<usize>();
        let total_plays = total_plays - total_plays_before;
        dbg!(total_plays);

        let sims_per_sec = total_plays as f64 / (elapsed.as_millis() as f64 / 1_000_f64);
        println!("Simulations per sec: {:.0}", sims_per_sec);

        for action_result in &results {
            let sat_display = if action_result.is_saturated {
                "(S)"
            } else {
                ""
            };

            println!(
                "Action: {:?} Plays: {} Wins: {} ({:.2}) {}",
                action_result.action,
                action_result.plays,
                action_result.wins,
                action_result.wins as f32 / action_result.plays as f32,
                sat_display,
            );
        }
    }

    if results.iter().all(|r| r.is_saturated) {
        results
            .iter()
            .max_by_key(|r| (r.wins * 10000) / r.plays)
            .expect("Must have been a max result")
            .clone()
    } else {
        results
            .iter()
            .max_by_key(|r| r.plays)
            .expect("Must have been a max result")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lib_boardgame::{Game, GameState};
    use lib_tic_tac_toe::tic_tac_toe::TicTacToe;
    use lib_tic_tac_toe::tic_tac_toe_gamestate::{BoardPosition, TicTacToeAction};

    #[test]
    fn tree_search_always_picks_winning_move() {
        let black_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let white_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::White);

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

        let test_black_agent: MctsAgent<_, ArcNode<_>> = MctsAgent::new(PlayerColor::Black);
        let mcts_chosen_move = test_black_agent.pick_move(state, &legal_moves);

        // The agent MUST pick the winning move:
        //  V
        // XXX
        // _O_
        // __O
        assert_eq!(TicTacToeAction(BoardPosition::new(1, 2)), mcts_chosen_move);
    }
}
