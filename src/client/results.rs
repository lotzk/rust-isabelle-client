/// Contains the result data types the Isabelle servers responses with
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
    canceled: bool,
    consolidated: bool,
    percentage: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodesStatus {
    status: Vec<(Node, NodeStatus)>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Export {
    name: String,
    base64: bool,
    body: String,
}

/// Results per sessions for `session_build` command
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionBuildResult {
    /// The target session name as specified by the command
    pub session: String,
    /// True if building was successful
    pub ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    pub return_code: usize,
    /// If true, the build process was aborted after running too long
    timeout: bool,
    /// Overall timing
    timing: Timing,
}

/// Results for `session_build` command
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionBuildResults {
    /// All sessions ok
    pub ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    return_code: usize,
    /// The result of each build sessions
    pub sessions: Vec<SessionBuildResult>,
}

/// Results for `session_start` command
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStartResult {
    pub task: String,
    /// Internal identification of the session object within the server process
    pub session_id: String,
    /// Temporary directory that is specifically cre- ated for this session and deleted after it has been stopped.
    /// As tmp_dir is the default master_dir for commands use_theories and purge_theories, theory files copied there may be used without further path specification.
    pub tmp_dir: Option<String>,
}

/// Results for `session_stop` command
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopResult {
    pub task: String,
    pub ok: bool,
    pub return_code: usize,
}

/// Result per node as returned from the `use_theories` command
#[derive(Deserialize, Serialize, Debug)]
pub struct NodeResults {
    #[serde(flatten)]
    node: Node,
    status: NodeStatus,
    messages: Vec<Message>,
    exports: Vec<Export>,
}

/// Results for `use_theories` command
#[derive(Deserialize, Serialize, Debug)]
pub struct UseTheoryResults {
    pub task: String,
    pub ok: bool,
    pub errors: Vec<Message>,
    pub nodes: Vec<NodeResults>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PurgedTheory {
    pub node_name: String,
    pub theory_name: String,
}

/// Results for `purge_theories` command.
///
/// The system manual states that the result is of the form `{purged: [String]}`, which is incorrect.
/// The struct models what is actually returned by the server.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PurgeTheoryResults {
    pub purged: Vec<PurgedTheory>,
    pub retained: Vec<PurgedTheory>,
}
