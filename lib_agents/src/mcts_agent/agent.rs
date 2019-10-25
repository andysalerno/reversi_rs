use super::tree_search_par;
use lib_boardgame::{GameAgent, GameState, PlayerColor};
use lib_printer::{out, out_impl};
use monte_carlo_tree::{
    amonte_carlo_data::AMctsData, amonte_carlo_data::MctsResult, arc_tree::ArcNode, tree::Node,
};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::Sync;
use std::time::Instant;

pub struct MctsAgent<TState, TNode = ArcNode<AMctsData<TState>>>
where
    TState: GameState,
    TNode: Node<Data = AMctsData<TState>>,
{
    color: PlayerColor,
    current_state_root: RefCell<Option<TNode::Handle>>,
    anticipated_opponent_actions: RefCell<Vec<TState::Move>>,
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
            anticipated_opponent_actions: Default::default(),
        }
    }

    fn walk_tree_to_child(&self, action: TState::Move) {
        // IDEA: half threads are "win seekers" and other half is "loss seeker"
        // (i.e. explores as though we're playing for the opponent)
        let root_handle = self
            .current_root_handle()
            .expect("Must have a root node to seek through.");
        let cur_state_node = root_handle.borrow();
        let children = cur_state_node.children_read();

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
    TNode: Node<Data = AMctsData<TState>>,
    TState: GameState + Sync,
{
    fn observe_action(&self, player: PlayerColor, action: TState::Move, _result: &TState) {
        // TODO: this might get broken by skipped turns
        if player == self.color.opponent()
            && self
                .anticipated_opponent_actions
                .borrow()
                .iter()
                .take(3)
                .find(|&&a| a == action)
                .is_none()
        {
            out!(
                "Player {:?} didn't expect action: {:?}. Expected action in: {:?}",
                self.color,
                action,
                self.anticipated_opponent_actions.borrow()
            );
        }

        if self.current_root_handle().is_some() {
            self.walk_tree_to_child(action);
        }
    }

    fn pick_move(&self, state: &TState, _legal_moves: &[TState::Move]) -> TState::Move {
        let root_handle = self
            .current_root_handle()
            .unwrap_or_else(|| self.reset_root_handle(state));
        let copy_handle = root_handle.clone();

        let result = match self.color {
            PlayerColor::Black => perform_mcts_par::<TNode, TState>(root_handle, self.color),
            PlayerColor::White => perform_mcts_par::<TNode, TState>(root_handle, self.color),
        };

        // Find the anticipated opponent responses
        {
            let our_selected_child = copy_handle
                .borrow()
                .children_read()
                .iter()
                .cloned()
                .find(|n| n.borrow().data().action().unwrap() == result.action)
                .unwrap();

            let mut opponent_choices = our_selected_child.borrow().children_read().clone();

            opponent_choices.sort_by_key(|c| c.borrow().data().plays());

            let mut anticipated = self.anticipated_opponent_actions.borrow_mut();
            anticipated.drain(..);

            let sum_plays: usize = opponent_choices
                .iter()
                .map(|c| c.borrow().data().plays())
                .sum();

            for c in opponent_choices.iter().rev() {
                let data = c.borrow().data();

                out!(
                    "Anticipated response: {:?} wins/plays: {:?}/{:?} ({:.3})",
                    data.action().expect("The choice had no action available."),
                    data.wins(),
                    data.plays(),
                    data.plays() as f32 / sum_plays as f32
                );
                anticipated.push(data.action().expect("Must have had an action."));
            }
        }

        let white_wins = if self.color == PlayerColor::White {
            result.wins
        } else {
            result.plays - result.wins
        };

        out!("{}", pretty_ratio_bar_text(20, white_wins, result.plays));

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
) -> MctsResult<TState>
where
    TNode: Node<Data = AMctsData<TState>> + Sync,
    TState: GameState + Sync,
{
    let total_plays_before = root
        .borrow()
        .children_read()
        .iter()
        .map(|c| c.borrow().data().plays())
        .sum::<usize>();

    let now = Instant::now();
    let results = tree_search_par::mcts_result::<TNode, TState>(root, player_color);
    let elapsed = now.elapsed();

    // Some friendly UI output
    {
        let total_plays = results.iter().map(|r| r.plays).sum::<usize>();
        let total_plays = total_plays - total_plays_before;

        let sims_per_sec = total_plays as f64 / (elapsed.as_millis() as f64 / 1_000_f64);
        out!("Simulations per sec: {:.0}", sims_per_sec);

        for action_result in &results {
            out!("{:?}", action_result);
        }
    }

    if results.iter().all(|r| r.is_saturated) {
        let mut results = results.iter().cloned().collect::<Vec<_>>();
        results.sort_by_key(|r| r.worst_wins / r.worst_plays);

        results.pop().unwrap()
    } else {
        results
            .iter()
            .max_by_key(|r| r.plays)
            .expect("Must have been a max result")
            .clone()
        // }
        // TODO experimenting with this
        // results
        //     .iter()
        //     .max_by_key(|r| (r.wins * 10000) / r.plays)
        //     .expect("Must have been a max result")
        //     .clone()
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
        let black_agent: Box<MctsAgent<_, ArcNode<_>>> =
            Box::new(MctsAgent::new(PlayerColor::Black));
        let white_agent: Box<MctsAgent<_, ArcNode<_>>> =
            Box::new(MctsAgent::new(PlayerColor::White));

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
