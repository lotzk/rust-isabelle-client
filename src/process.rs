use std::{
    collections::HashMap,
    io,
    path::PathBuf,
    process::{Output, Stdio},
};
use tokio::process::Command;

/// Arguments for running the raw ML process in batch mode.
#[derive(Default)]
pub struct ProcessArgs {
    /// The theories to load (-T). Multiple theories are loaded in the given order.
    pub theories: Vec<String>,
    // Include session directories (-d)
    pub session_dirs: Vec<String>,
    /// The The logic session name (default is ISABELLE_LOGIC="HOL")
    pub logic: Option<String>,
    /// Override Isabelle system options for this process (-d).
    /// Use [OptionsBuilder] to construct options.
    pub options: HashMap<String, String>,
}

impl ProcessArgs {
    pub fn load_theories(ths: &[String]) -> Self {
        Self {
            theories: ths.to_vec(),
            ..Default::default()
        }
    }
}

/// Runs the raw ML process in batch mode.
/// Arguments for the command are specified in [ProcessArgs].
/// Returns the process' output.
///
/// # Example
///
/// ```rust
/// use isabelle_client::process::{batch_process, ProcessArgs};
/// use tokio_test;
/// # tokio_test::block_on(async {
///
///
/// let args = ProcessArgs::load_theories(&[String::from("~~/src/HOL/Examples/Drinker")]);
/// let output = batch_process(&args, None).await;
/// assert!(output.unwrap().status.success());
/// })
/// ```
pub async fn batch_process(
    args: &ProcessArgs,
    current_dir: Option<&PathBuf>,
) -> io::Result<Output> {
    let mut isabelle_cmd = Command::new("isabelle");

    isabelle_cmd
        .arg("process")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());

    if let Some(cd) = current_dir {
        isabelle_cmd.current_dir(cd);
    }

    for t in &args.theories {
        isabelle_cmd.arg("-T").arg(t);
    }
    for d in &args.session_dirs {
        isabelle_cmd.arg("-d").arg(d);
    }

    for (k, v) in args.options.iter() {
        isabelle_cmd.arg("-o").arg(format!("{}={}", k, v));
    }

    isabelle_cmd.spawn()?.wait_with_output().await
}

/// Builder that conveniently allows to specify common Isabelle options.
#[derive(Default)]
pub struct OptionsBuilder {
    options: HashMap<String, String>,
}

impl From<OptionsBuilder> for HashMap<String, String> {
    /// Consumes the builder and return a key value map of the specified options.
    fn from(val: OptionsBuilder) -> Self {
        val.options
    }
}

impl OptionsBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn add_int_option(&mut self, k: &str, v: isize) -> &mut Self {
        self.options.insert(k.to_owned(), v.to_string());
        self
    }

    fn add_bool_option(&mut self, k: &str, v: bool) -> &mut Self {
        self.options.insert(k.to_owned(), v.to_string());
        self
    }

    fn add_real_option(&mut self, k: &str, v: f32) -> &mut Self {
        self.options
            .insert(k.to_owned(), v.to_string().to_lowercase());
        self
    }

    /// Maximum number of worker threads for prover process (0 = hardware max.)
    pub fn threads(&mut self, threads: isize) -> &mut Self {
        self.add_int_option("threads", threads)
    }

    /// Maximum stack size for worker threads (in giga words, 0 = unlimited)
    pub fn thread_stack_limit(&mut self, limit: f32) -> &mut Self {
        self.add_real_option("threads_stack_limit", limit)
    }

    /// Approximative limit for parallel tasks (0 = unlimited)
    pub fn parallel_limit(&mut self, limit: isize) -> &mut Self {
        self.add_int_option("parallel_limit", limit)
    }

    /// Level of parallel proof checking: 0, 1, 2
    pub fn parallel_proofs(&mut self, limit: isize) -> &mut Self {
        self.add_int_option("parallel_proofs", limit)
    }

    /// Scale factor for timeout in Isabelle/ML and session build
    pub fn timeout_scale(&mut self, scale: f32) -> &mut Self {
        self.add_real_option("timeout_scale", scale)
    }

    // "Detail of Proof Checking"

    /// Level of proofterm recording: 0, 1, 2, negative means unchanged
    pub fn record_proofs(&mut self, level: isize) -> &mut Self {
        self.add_int_option("record_proofs", level)
    }

    /// If true then some tools will OMIT some proofs
    pub fn quick_and_dirty(&mut self, flag: bool) -> &mut Self {
        self.add_bool_option("quick_and_dirty", flag)
    }

    /// If true then skips over proofs (implicit 'sorry')
    pub fn skip_proofs(&mut self, flag: bool) -> &mut Self {
        self.add_bool_option("skip_proofs", flag)
    }

    // "Global Session Parameters"

    /// Timeout for session build job (seconds > 0)
    pub fn timeout(&mut self, limit: f32) -> &mut Self {
        self.add_real_option("timeout", limit)
    }

    // Observe timeout for session build (default 'true')
    pub fn timeout_build(&mut self, flag: bool) -> &mut Self {
        self.add_bool_option("timeout_build", flag)
    }

    /// Build process output limit (in million characters, 0 = unlimited)
    pub fn process_output_limit(&mut self, limit: isize) -> &mut Self {
        self.add_int_option("process_output_limit", limit)
    }

    /// Build process output tail shown to user (in lines, 0 = unlimited)
    pub fn process_output_tail(&mut self, tail: isize) -> &mut Self {
        self.add_int_option("process_output_tail", tail)
    }

    // "PIDE Build"

    // Report PIDE markup (in ML) (default 'true')
    pub fn pide_reports(&mut self, flag: bool) -> &mut Self {
        self.add_bool_option("pide_reports", flag)
    }

    // Report PIDE markup (in batch build) (default 'true')
    pub fn build_pide_reports(&mut self, flag: bool) -> &mut Self {
        self.add_bool_option("build_pide_reports", flag)
    }
}
