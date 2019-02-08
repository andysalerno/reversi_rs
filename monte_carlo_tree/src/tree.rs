use std::borrow::Borrow;

pub trait Node
where
    Self: Sized,
{
    type ChildrenIter: IntoIterator<Item = Self>;
    type ParentBorrow: Borrow<Self>;
    type Data;

    fn data(&self) -> &Self::Data;
    fn parent(&self) -> Option<Self::ParentBorrow>;
    fn children(&self) -> Self::ChildrenIter;
    fn add_child(&mut self, child: Self);

    fn new_child(&self, state: &Self::Data) -> Self;
    fn new_root(state: Self::Data) -> Self;
}
