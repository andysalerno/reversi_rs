use crate::Node;
use std::borrow::Borrow;
use std::fmt::Display;

pub trait TreeToDotFileFormat {
    fn to_dot_file_str(&self) -> String;
}

impl<T> TreeToDotFileFormat for T
where
    T: Node,
    T::Data: Display
{
    fn to_dot_file_str(&self) -> String {
        const HEADER: &str = "digraph prof {\nratio = fill;\nnode [style=filled];";
        const FOOTER: &str = "}";

        let mut node_mappings = String::new();

        let mut node_handles_queue = vec![self.get_handle()];

        while let Some(cur_handle) = node_handles_queue.pop() {
            let node = cur_handle.borrow();
            let node_repr = format!("{}", node.data());

            for c in node.children() {
                let child_node = c.borrow();
                let child_repr = format!("{}", child_node.data());
                let dot_format = format!("{} -> {}\n", node_repr, child_repr);

                node_mappings.push_str(&dot_format);

                node_handles_queue.push(c);
            }
        }

        format!("{}{}{}", HEADER, node_mappings, FOOTER)
    }
}
