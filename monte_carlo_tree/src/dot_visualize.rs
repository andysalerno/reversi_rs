use crate::monte_carlo_data::MctsData;
use crate::tree::Node;
use lib_boardgame::GameState;
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait TreeToDotFileFormat {
    fn to_dot_file_str(&self, depth_limit: Option<usize>) -> String;
}

impl<T, TState> TreeToDotFileFormat for T
where
    T: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    fn to_dot_file_str(&self, depth_limit: Option<usize>) -> String {
        const HEADER: &str =
            "digraph prof {\nratio = fill;\nnode [style=filled, shape=box, fontname=courier];\n";
        const FOOTER: &str = "}";

        let mut node_id_map_buf = String::new();
        let mut node_labels_buf = String::new();

        depth_first_tree_walk(
            self,
            0,
            &mut node_labels_buf,
            &mut node_id_map_buf,
            depth_limit,
        );

        format!("{}{}{}{}", HEADER, node_labels_buf, node_id_map_buf, FOOTER)
    }
}

fn depth_first_tree_walk<T, TState>(
    node: &T,
    path_hash: u64,
    node_labels_buf: &mut String,
    node_id_map_buf: &mut String,
    depth_remaining: Option<usize>,
) where
    T: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    if let Some(d) = depth_remaining {
        if d == 0 {
            return;
        }
    }

    let label = node_label(node);
    let id = hash_str(&label).wrapping_add(path_hash);

    let label_str = format!("{} [label = \"{}\"]\n", id, label);

    // Add the label for this node
    node_labels_buf.push_str(&label_str);

    if depth_remaining.is_none() || depth_remaining.unwrap() > 1 {
        for child in node.children() {
            let child_label = node_label(child.borrow());
            let child_id = hash_str(&child_label).wrapping_add(id);

            if depth_remaining.is_none() {
                // We're not recursing anymore, so we need to define child labels here
                let label_str = format!("{} [label = \"{}\"]\n", child_id, child_label);
                node_labels_buf.push_str(&label_str);
            }

            let id_mapping_str = format!("{} -> {};\n", id, child_id);

            // Add the mapping from this node to the child node
            node_id_map_buf.push_str(&id_mapping_str);

            if depth_remaining.is_some() {
                depth_first_tree_walk(
                    child.borrow(),
                    id,
                    node_labels_buf,
                    node_id_map_buf,
                    depth_remaining.and_then(|v| Some(v - 1)),
                );
            }
        }
    }

    if let Some(d) = depth_remaining {
        let max_child = node
            .children()
            .into_iter()
            .max_by_key(|n| n.borrow().data().plays());

        if d == 1 && max_child.is_some() {
            // If we're the very last layer, add one more layer for good measure for the best option
            depth_first_tree_walk(
                max_child.unwrap().borrow(),
                id,
                node_labels_buf,
                node_id_map_buf,
                None,
            );
        }
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

fn node_label<T, TState>(node: &T) -> String
where
    T: Node<Data = MctsData<TState>>,
    TState: GameState,
{
    let data = node.data();
    let action_str = match data.action() {
        Some(a) => format!("{}", a),
        None => "n/a".into(),
    };

    let label = format!(
        "A: {}\nWins: {}\nPlays: {}\n{}",
        action_str,
        data.wins(),
        data.plays(),
        data
    );

    sanitize_newlines(label)
}
