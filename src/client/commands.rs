use serde::{Deserialize, Serialize};

/// Arguments for `cancel` command
#[derive(Deserialize, Serialize, Debug)]
pub struct CancelArgs {
    /// Id of the task to try to cancel
    pub task: String,
}

/// Arguments for `session_build` and `session_start` commands
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
    pub include_sessions: Vec<String>,
}

impl SessionBuildArgs {
    pub fn session(session: &str) -> Self {
        Self {
            session: session.to_string(),
            ..Default::default()
        }
    }
}

/// Arguments for `session_stop` command
#[derive(Deserialize, Serialize, Debug)]
pub struct SessionStopArgs {
    /// Id of the session to stop
    pub session_id: String,
}

/// Arguments for `use_theories` command
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UseTheoriesArgs {
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

impl UseTheoriesArgs {
    pub fn for_session(session_id: &str, theories: &[&str]) -> Self {
        Self {
            session_id: session_id.to_string(),
            theories: theories.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }
    }
}

/// Arguments for `purge_theories` command
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
