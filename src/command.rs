use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::common::*;

pub trait ClientCommand<T> {}
pub enum Args<T: serde::Serialize> {
    Json(T),
    String(String),
    None,
}
pub struct Command<T: serde::Serialize> {
    pub name: String,
    pub args: Args<T>,
}

pub enum SyncResponse {
    Ok(String),
    Error(String),
}

#[derive(Debug)]
pub enum SyncResult<T, E> {
    Ok(T),
    Error(E),
}

#[derive(Debug)]
pub enum AsyncResult<T, E, F> {
    Error(E),
    Finished(T),
    Failed(FailedResult<F>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedResult<T> {
    task: Task,
    error_message: String,

    #[serde(flatten)]
    context: T,
}

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
pub struct SessionBuildArgs {
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
    pub include_session: Vec<String>,
}

impl SessionBuildArgs {
    pub fn session(session: &str) -> Self {
        Self {
            session: session.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionBuildResult {
    task: String,
    /// The target session name as specified by the command
    session: String,
    /// True if building was successful
    ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    return_code: usize,
    /// If true, the build process was aborted after running too long
    timeout: bool,
    /// Overall timing
    timing: Timing,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionBuildResults {
    /// All sessions ok
    ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    return_code: usize,
    /// The result of each build sessions
    sessions: Vec<SessionBuildResult>,
}

/// Starts a new Isabelle/PIDE session with un- derlying Isabelle/ML process, based on a session image that it produces on demand using session_build.
/// Sessions are independent of client connections: it is possible to start a session and later apply `use_theories` on different connections, as long as the internal session identifier is known.
/// Shared theory imports will be used only once (and persist until purged explicitly).
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStartArgs {
    /// The target session name as specified by the command
    session: String,
    /// True if building was successful
    ok: bool,
    /// Is zero if `ok` is true. Non-zero return code indicates and error.
    return_code: usize,
    /// If true, the build process was aborted after running too long
    timeout: bool,
    /// Overall timing
    timing: Timing,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStartResult {
    task: String,
    /// Internal identification of the session object within the server process
    session_id: String,
    /// Temporary directory that is specifically cre- ated for this session and deleted after it has been stopped.
    /// As tmp_dir is the default master_dir for commands use_theories and purge_theories, theory files copied there may be used without further path specification.
    tmp_dir: Option<String>,
}

/// Forces a shutdown of the identified session.
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopArgs {
    /// Id of the session to stop
    session_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopResult {
    task: String,
    ok: bool,
    return_code: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UseTheoryArgs {
    session_id: String,
    theories: Vec<String>,
    master_dir: Option<String>,
    unicode_symbols: Option<bool>,
    export_pattern: Option<String>,
    check_delay: Option<f64>,
    check_limit: Option<usize>,
    watchdog_timeout: Option<f64>,
    nodes_status_delay: Option<f64>,
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
    status: NodeStatus,
    messages: Vec<Message>,
    exports: Vec<Export>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UseTheoryResults {
    task: String,
    ok: bool,
    errors: Vec<Message>,
    nodes: Vec<(Node, NodeResults)>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PurgeTheoryArgs {}

#[derive(Deserialize, Serialize, Debug)]
pub struct PurgeTheoryResult {}
