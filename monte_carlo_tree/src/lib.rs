mod rc_tree;

trait Node<'a> {
    fn plays(&self) -> usize;
    fn wins(&self) -> usize;
    fn losses(&self) -> usize;
    fn parent(&self) -> &'a Self;
    fn parent_mut(&mut self) -> &'a mut Self;
    fn children(&self) -> &'a [Self]
    where
        Self: Sized;
}

trait MonteCarloTree {
    fn select_child();
    fn simulate();
    fn backprop();
}
