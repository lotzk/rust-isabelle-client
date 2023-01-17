use serde::{Deserialize, Serialize};

/// Describes a source position within Isabelle text
#[derive(Deserialize, Serialize, Debug)]
struct Position {
    line: Option<usize>,
    offset: Option<usize>,
    end_offset: Option<usize>,
    filed: Option<String>,
    id: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    /// The main message kinds are writeln (for regular output), warning, error.
    kind: String,
    message: String,
    pos: Option<Position>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ErrorMessage {
    /// = "error"
    kind: String,
    message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TheoryProgress {
    /// = "writeln"
    kind: String,
    message: String,
    session: String,
    percentage: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Timing {
    elapsed: f64,
    cpu: f64,
    gc: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    task: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Node {
    node_name: String,
    theory_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeStatus {
    ok: bool,
    total: usize,
    unprocessed: usize,
    running: usize,
    warned: usize,
    failed: usize,
    cancelled: bool,
    consolidated: bool,
    percentage: bool,
}
