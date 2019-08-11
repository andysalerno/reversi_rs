use crate::Node;
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

        let mut node_id_mappings = String::new();
        let mut node_labels = String::new();

        let mut node_handles_queue = vec![self.get_handle()];

        while let Some(cur_handle) = node_handles_queue.pop() {
            let node = cur_handle.borrow();
            let n_repr = node_repr(node);
            let n_id = node_id(node);

            for c in node.children() {
                // Every node's unique ID will be hash(parent_repr) + hash(node_repr)
                // This is to avoid creating nodes with multiple parents
                // (e.g. if both State_X and State_Y have some action that brings them to an identical State_Z)
                let child_node = c.borrow();
                let child_id = node_id(child_node);

                let node_id_mapping = format!("{} -> {};\n", n_id, child_id);

                node_id_mappings.push_str(&node_id_mapping);

                node_handles_queue.push(c);
            }

            let node_label = format!("{} [label = \"{}\"]\n", n_id, n_repr);
            node_labels.push_str(&node_label);
        }

        format!("{}{}{}{}", HEADER, node_labels, node_id_mappings, FOOTER)
    }
}

fn depth_first_tree_walk<T>(node: &T, path_hash: u64, node_labels_buf: &mut String, node_id_map_buf: &mut String)
where
    T: Node,
    T::Data: Display,
{
    let label = node_label(node);
    let id = hash_str(label) + path_hash;

    let label_str = format!("{} [label = \"{}\"]\n", id, label);
    node_labels_buf.push_str(&label_str);

    for child in node.children() {
        let child_label = node_label(child.borrow());
        let child_id = hash_str(child_label) + id;

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
    T::Data: Display,
{
    sanitize_newlines(format!("{}", node.data()))
}

fn node_id<T>(node: &T) -> u64
where
    T: Node,
    T::Data: Display,
{
    // node_id is the combined hash of its parent repr and its own repr
    let path_to_parent = Vec::new();

    let mut node_walker = Some(node.get_handle());
    
    while let Some(cur_node) = node_walker {
        let node_str = node_repr(cur_node.borrow());
    }



    let child_repr = node_repr(node);

    let parent_repr = if let Some(parent) = node.parent() {
        node_repr(parent.borrow())
    } else {
        String::new()
    };

    hash_str(child_repr) + hash_str(parent_repr)
}
