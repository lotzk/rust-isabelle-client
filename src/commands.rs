use serde::{Deserialize, Serialize};

use crate::common::*;

/// Attempts to cancel the task with the specified task id.
/// May get ignored by the running task.
#[derive(Deserialize, Serialize, Debug)]
pub struct CancelArgs {
    /// Id of the task to try to cancel
    pub task: String,
}

/// Prepares a session image for interactive use of theories.
/// The build process is asynchronous, with notifications that inform about the progress of loaded theories.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct SessionBuildStartArgs {
    /// Specifies the target session name. The build process will produce all required ancestor images according to the overall session graph.
    pub session: String,
    /// Environment of Isabelle system options is determined from preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<String>,
    /// List of individual updates to the Isabelle system environment of the form `name=value` or `name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    /// Specifies additional directories for session `ROOT`and `ROOTS` files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirs: Option<Vec<String>>,
    /// Specifies sessions whose theories should be included in the overall name space of session-qualified theory names.
    /// Corresponds to `session` specification in `ROOT` files.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub include_sessions: Vec<String>,
}

impl SessionBuildStartArgs {
    pub fn session(session: &str) -> Self {
        Self {
            session: session.to_string(),
            ..Default::default()
        }
    }
}

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

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionBuildResults {
    /// All sessions ok
    pub ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    return_code: usize,
    /// The result of each build sessions
    pub sessions: Vec<SessionBuildResult>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStartResult {
    pub task: String,
    /// Internal identification of the session object within the server process
    pub session_id: String,
    /// Temporary directory that is specifically cre- ated for this session and deleted after it has been stopped.
    /// As tmp_dir is the default master_dir for commands use_theories and purge_theories, theory files copied there may be used without further path specification.
    pub tmp_dir: Option<String>,
}

/// Forces a shutdown of the identified session.
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopArgs {
    /// Id of the session to stop
    pub session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopResult {
    pub task: String,
    pub ok: bool,
    pub return_code: usize,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UseTheoryArgs {
    pub session_id: String,
    pub theories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unicode_symbols: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_delay: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchdog_timeout: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes_status_delay: Option<f64>,
}

impl UseTheoryArgs {
    pub fn for_session(session_id: &str, theories: &[&str]) -> Self {
        Self {
            session_id: session_id.to_string(),
            theories: theories.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }
    }
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

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeResults {
    #[serde(flatten)]
    node: Node,
    status: NodeStatus,
    messages: Vec<Message>,
    exports: Vec<Export>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UseTheoryResults {
    pub task: String,
    pub ok: bool,
    pub errors: Vec<Message>,
    pub nodes: Vec<NodeResults>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PurgeTheoryArgs {
    pub session_id: String,
    pub theories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
}

impl PurgeTheoryArgs {
    pub fn for_session(session_id: &str, theories: &[&str]) -> Self {
        Self {
            session_id: session_id.to_string(),
            theories: theories.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }
    }
}

/// The system manual states that the result is of the form `{purged: [String]}`, which is incorrect.
/// The struct models what is actually returned by the server.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PurgeTheoryResults {
    pub purged: Vec<PurgedTheory>,
    pub retained: Vec<PurgedTheory>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PurgedTheory {
    pub node_name: String,
    pub theory_name: String,
}
