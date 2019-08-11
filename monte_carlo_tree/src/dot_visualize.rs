use crate::tree::Node;
use crate::monte_carlo_data::MctsData;
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

pub trait TreeToDotFileFormat {
    fn to_dot_file_str(&self) -> String;
}

impl<T> TreeToDotFileFormat for T
where
    T: Node,
    T::Data: Display,
{
    fn to_dot_file_str(&self) -> String {
        const HEADER: &str = "digraph prof {\nratio = fill;\nnode [style=filled];\n";
        const FOOTER: &str = "}";

        let mut node_id_map_buf = String::new();
        let mut node_labels_buf = String::new();

        depth_first_tree_walk(self, 0, &mut node_labels_buf, &mut node_id_map_buf);

        format!("{}{}{}{}", HEADER, node_labels_buf, node_id_map_buf, FOOTER)
    }
}

fn depth_first_tree_walk<T>(node: &T, path_hash: u64, node_labels_buf: &mut String, node_id_map_buf: &mut String)
where
    T: Node,
    T::Data: Display,
{
    let label = node_label(node);
    let id = hash_str(&label).wrapping_add(path_hash);

    let label_str = format!("{} [label = \"{}\"]\n", id, label);
    node_labels_buf.push_str(&label_str);

    for child in node.children() {
        let child_label = node_label(child.borrow());
        let child_id = hash_str(child_label).wrapping_add(id);

        let id_mapping_str = format!("{} -> {};\n", id, child_id);

        node_id_map_buf.push_str(&id_mapping_str);

        depth_first_tree_walk(child.borrow(), id, node_labels_buf, node_id_map_buf);
    }
}

fn sanitize_newlines<T: AsRef<str>>(s: T) -> String {
    s.as_ref().replace("\n", "\\n")
}

fn hash_str<T: AsRef<str>>(s: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.as_ref().hash(&mut hasher);
    hasher.finish()
}

fn node_label<T>(node: &T) -> String
where
    T: Node,
    T::Data: Display
{
    // let data = node.data();
    // let label = format!("{}\n{}\n{}", data.wins(), data.plays(), data);
    let label = format!("{}", node.data());
    sanitize_newlines(label)
}
