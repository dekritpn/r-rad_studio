// src-tauri/src/r_daemon.rs
// Persistent R Daemon lifecycle manager with asynchronous JSON-RPC communication

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
pub struct RDaemonConfig {
    pub custom_rscript_path: Option<PathBuf>,
    pub bridge_script_path: PathBuf,
}

impl Default for RDaemonConfig {
    fn default() -> Self {
        let bridge_path = find_bridge_script();
        Self {
            custom_rscript_path: None,
            bridge_script_path: bridge_path,
        }
    }
}

pub fn find_rscript_binary(custom_path: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(cp) = custom_path {
        if cp.exists() {
            return Ok(cp.to_path_buf());
        }
    }

    // 1. Check local environment directory (e.g. micromamba)
    if let Ok(home) = std::env::var("HOME") {
        let local_env_r = PathBuf::from(&home).join(".local/share/r-rad-env/bin/Rscript");
        if local_env_r.exists() {
            return Ok(local_env_r);
        }
    }

    // 2. Check system PATH via `which`
    if let Ok(path) = which::which("Rscript") {
        return Ok(path);
    }

    // 3. Fallback known system locations
    let common_locations = [
        "/usr/bin/Rscript",
        "/usr/local/bin/Rscript",
        "/opt/homebrew/bin/Rscript",
    ];
    for loc in common_locations {
        let p = PathBuf::from(loc);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("Could not locate Rscript binary on system. Please ensure R is installed.".to_string())
}

pub fn find_bridge_script() -> PathBuf {
    // Check relative paths from current working dir or executable dir
    let candidates = [
        PathBuf::from("r-runtime/bridge.R"),
        PathBuf::from("../r-runtime/bridge.R"),
        PathBuf::from("../../r-runtime/bridge.R"),
    ];

    for c in &candidates {
        if c.exists() {
            return std::fs::canonicalize(c).unwrap_or_else(|_| c.clone());
        }
    }

    // Fallback to absolute workspace path if available
    let workspace_bridge = PathBuf::from("/home/dpng/Projects/r_rad_studio/r-runtime/bridge.R");
    if workspace_bridge.exists() {
        return workspace_bridge;
    }

    PathBuf::from("r-runtime/bridge.R")
}

pub struct RProcessInstance {
    pub child: Child,
    pub stdin: ChildStdin,
    pub lines: Lines<BufReader<ChildStdout>>,
    pub r_version: String,
}

pub struct RDaemon {
    config: RDaemonConfig,
    instance: Arc<Mutex<Option<RProcessInstance>>>,
}

impl RDaemon {
    pub fn new(config: RDaemonConfig) -> Self {
        Self {
            config,
            instance: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn ensure_started(&self) -> Result<(), String> {
        let mut guard = self.instance.lock().await;
        if guard.is_none() {
            let instance = self.spawn_process().await?;
            *guard = Some(instance);
        }
        Ok(())
    }

    async fn spawn_process(&self) -> Result<RProcessInstance, String> {
        let rscript = find_rscript_binary(self.config.custom_rscript_path.as_deref())?;
        let bridge = &self.config.bridge_script_path;

        if !bridge.exists() {
            return Err(format!(
                "bridge.R script not found at path: {}",
                bridge.display()
            ));
        }

        let mut cmd = Command::new(&rscript);
        cmd.arg("--vanilla")
            .arg(bridge)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn R process: {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture R stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture R stdout".to_string())?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Read startup READY message
        let ready_line = tokio::time::timeout(std::time::Duration::from_secs(10), lines.next_line())
            .await
            .map_err(|_| "Timeout waiting for R daemon to start".to_string())?
            .map_err(|e| format!("Error reading from R daemon: {e}"))?
            .ok_or_else(|| "R process exited prematurely before sending READY signal".to_string())?;

        let ready_json: serde_json::Value = serde_json::from_str(&ready_line)
            .map_err(|e| format!("Failed to parse R startup payload: {e} (Raw: {ready_line})"))?;

        let r_version = ready_json["version"]
            .as_str()
            .unwrap_or("Unknown R")
            .to_string();

        Ok(RProcessInstance {
            child,
            stdin,
            lines,
            r_version,
        })
    }

    pub async fn send_request(
        &self,
        command: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.ensure_started().await?;

        let req_id = Uuid::new_v4().to_string();
        let payload = serde_json::json!({
            "id": req_id,
            "command": command,
            "args": args
        });

        let mut json_str = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;
        json_str.push('\n');

        let mut guard = self.instance.lock().await;
        let proc = guard
            .as_mut()
            .ok_or_else(|| "R instance not running".to_string())?;

        proc.stdin
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to R stdin: {e}"))?;
        proc.stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush R stdin: {e}"))?;

        let resp_line = proc
            .lines
            .next_line()
            .await
            .map_err(|e| format!("Failed to read response line from R: {e}"))?
            .ok_or_else(|| "R daemon closed stdout pipe unexpectedly".to_string())?;

        let resp_json: serde_json::Value = serde_json::from_str(&resp_line)
            .map_err(|e| format!("Failed to parse R JSON response: {e} (Raw: {resp_line})"))?;

        Ok(resp_json)
    }

    pub async fn execute_analysis(&self, r_code: &str) -> Result<serde_json::Value, String> {
        self.send_request("EXECUTE_ANALYSIS", serde_json::json!({ "r_code": r_code }))
            .await
    }

    pub async fn cancel(&self) -> Result<(), String> {
        let mut guard = self.instance.lock().await;
        if let Some(mut proc) = guard.take() {
            let _ = proc.child.kill().await;
        }
        // Respawn fresh instance
        let new_proc = self.spawn_process().await?;
        *guard = Some(new_proc);
        Ok(())
    }

    pub async fn restart(&self) -> Result<(), String> {
        self.cancel().await
    }
}
