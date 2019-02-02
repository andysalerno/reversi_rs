use lib_boardgame::game_primitives::GameState;
use std::cell::Cell;

pub mod rc_tree;

trait MakeNode<'a, T: GameState> {
    fn into_node(self) -> Node<'a, T>;
}

struct Data<T: GameState> {
    state: T,
    plays: Cell<usize>,
    wins: Cell<usize>,
    action: Option<T::Move>,
}

trait Node<'a, T: GameState>
where
    Self: Sized,
{
    fn data(&self) -> &Data<T>;
    fn parent(&self) -> Option<MakeNode<'a, T>>;
    fn children(&self) -> &[Self]
    where
        Self: Sized;
}

trait MonteCarloTree {
    fn select_child();
    fn expand();
    fn simulate();
    fn backprop();
}
