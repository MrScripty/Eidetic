//! Optional selected-root control-plane bootstrap. Pumas owns inference children.
use crate::pumas_inference::PumasClient;
use pumas_library::discovery::{CompatibilityRequirements, HttpServiceDescription, LocalDiscovery};
use serde::Deserialize;
use serde_json::json;
use std::{path::Path, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::Mutex,
    task::JoinHandle,
};

pub(crate) struct PumasRuntime {
    pub(crate) description: HttpServiceDescription,
    owner: Arc<Mutex<RuntimeOwner>>,
}

/// Child and output completion share the existing runtime lifecycle owner.
/// The composition root must still retain that owner through async cleanup.
#[derive(Default)]
struct RuntimeOwner {
    child: Option<Child>,
    stdout: Option<JoinHandle<std::io::Result<u64>>>,
    service_owned: bool,
    terminal_error: Option<String>,
}

#[derive(Deserialize)]
struct Bootstrap {
    bootstrap_schema_version: u32,
    ownership: String,
    description: HttpServiceDescription,
}

impl PumasRuntime {
    pub(crate) async fn from_environment() -> Result<Option<Arc<Self>>, String> {
        let Some(root) = std::env::var_os("EIDETIC_PUMAS_ROOT") else {
            return Ok(None);
        };
        let binary = std::env::var_os("EIDETIC_PUMAS_RPC_BIN");
        Self::attach(Path::new(&root), binary.as_deref().map(Path::new))
            .await
            .map(|runtime| Some(Arc::new(runtime)))
    }

    pub(crate) async fn attach(root: &Path, binary: Option<&Path>) -> Result<Self, String> {
        if let Some(binary) = binary {
            if !cfg!(target_os = "linux") {
                return Err("Pumas owned bootstrap is currently qualified on Linux only; run Pumas separately and borrow its endpoint".into());
            }
            let root = root.to_path_buf();
            let binary = binary.to_path_buf();
            let (reply, result) = tokio::sync::oneshot::channel();
            // Once admitted, construction/drain outlives a cancelled caller.
            tokio::spawn(async move {
                let runtime = Self::start(&root, &binary).await;
                if let Err(Ok(runtime)) = reply.send(runtime) {
                    if let Err(error) = runtime.shutdown().await {
                        tracing::error!(%error,"Cancelled Pumas startup cleanup failed");
                    }
                }
            });
            return result.await.map_err(|_| "Pumas startup task stopped")?;
        }
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let service = LocalDiscovery::open()
            .map_err(|e| e.to_string())?
            .borrow_http_service(&root, &CompatibilityRequirements::default())
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self {
            description: service.description().clone(),
            owner: Arc::new(Mutex::new(RuntimeOwner::default())),
        })
    }

    async fn start(root: &Path, binary: &Path) -> Result<Self, String> {
        let mut child = Command::new(binary)
            .arg("--attach-or-start-local-http")
            .arg("--launcher-root")
            .arg(root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Pumas bootstrap: {e}"))?;
        let stdout = child.stdout.take().ok_or("Pumas stdout unavailable")?;
        let mut reader = BufReader::new(stdout);
        let ack = tokio::time::timeout(Duration::from_secs(60), async {
            let mut total = 0;
            loop {
                let mut line = String::new();
                let count = reader
                    .read_line(&mut line)
                    .await
                    .map_err(|e| e.to_string())?;
                total += count;
                if count == 0 {
                    return Err("Pumas exited before local access acknowledgement".to_owned());
                }
                if total > 1024 * 1024 {
                    return Err("Pumas startup output exceeds limit".to_owned());
                }
                if let Some(value) = line.strip_prefix("PUMAS_LOCAL_ACCESS=") {
                    return serde_json::from_str::<Bootstrap>(value.trim())
                        .map_err(|e| e.to_string());
                }
            }
        })
        .await
        .map_err(|_| "Pumas startup timed out".to_owned())
        .and_then(|r| r);
        let ack = match ack {
            Ok(ack)
                if ack.bootstrap_schema_version == 1
                    && matches!(ack.ownership.as_str(), "owned" | "borrowed") =>
            {
                ack
            }
            result => {
                terminate_startup(&mut child).await?;
                return Err(result
                    .err()
                    .unwrap_or("Unsupported Pumas bootstrap acknowledgement".into()));
            }
        };
        // Retain completion and I/O errors with the child. Explicit shutdown
        // joins the drain; cancelled construction/last Drop still need the
        // composition-root owner described in the lifecycle acceptance report.
        let runtime = Self {
            description: ack.description,
            owner: Arc::new(Mutex::new(RuntimeOwner {
                child: Some(child),
                stdout: Some(tokio::spawn(async move {
                    tokio::io::copy(&mut reader, &mut tokio::io::sink()).await
                })),
                service_owned: ack.ownership == "owned",
                terminal_error: None,
            })),
        };
        if ack.ownership == "borrowed" {
            let mut owner = runtime.owner.lock().await;
            let child = owner.child.as_mut().expect("bootstrap child retained");
            let exit = match tokio::time::timeout(Duration::from_secs(10), child.wait()).await {
                Ok(result) => result.map_err(|e| format!("Borrowed Pumas bootstrap wait: {e}"))?,
                Err(_) => {
                    // This PID is only our spawned attach helper. The service
                    // identified by the borrowed acknowledgement is external;
                    // never send it shutdown/unload or signal its process.
                    child
                        .kill()
                        .await
                        .map_err(|e| format!("Borrowed Pumas helper termination: {e}"))?;
                    owner.child = None;
                    let output = finish_stdout(&mut owner).await;
                    return Err(match output {
                        Ok(()) => {
                            "Borrowed Pumas bootstrap did not exit; helper terminated and reaped"
                                .into()
                        }
                        Err(error) => {
                            format!("Borrowed Pumas bootstrap did not exit; helper reaped; {error}")
                        }
                    });
                }
            };
            owner.child = None;
            if !exit.success() {
                owner.terminal_error = Some(format!("Borrowed Pumas bootstrap exited with {exit}"));
            }
            finish_stdout(&mut owner).await?;
        }
        let client = PumasClient::connect(runtime.description.endpoint.as_str()).await;
        match client {
            Ok(client) if client.description == runtime.description => Ok(runtime),
            result => {
                runtime.shutdown().await?;
                Err(result
                    .err()
                    .unwrap_or("Pumas bootstrap generation changed".into()))
            }
        }
    }

    pub(crate) async fn shutdown(&self) -> Result<(), String> {
        shutdown_owner(&self.owner, &self.description).await
    }
}

async fn shutdown_owner(
    owner: &Mutex<RuntimeOwner>,
    description: &HttpServiceDescription,
) -> Result<(), String> {
    let mut owner = owner.lock().await;
    let service_owned = owner.service_owned;
    if let Some(child) = owner.child.as_mut() {
        let exit = if let Some(exit) = child.try_wait().map_err(|e| e.to_string())? {
            // An exited child has no shutdown authority over a replacement.
            exit
        } else if service_owned {
            let client = PumasClient::for_description(description.clone())?;
            tokio::time::timeout(Duration::from_secs(30), client.rpc("shutdown", json!({})))
                .await
                .map_err(|_| "Owned Pumas shutdown request timed out")??;
            tokio::time::timeout(Duration::from_secs(30), child.wait())
                .await
                .map_err(|_| "Owned Pumas drain timed out; child retained for retry")?
                .map_err(|e| e.to_string())?
        } else {
            // Failed borrowed bootstrap: terminate/reap our helper only.
            child
                .kill()
                .await
                .map_err(|e| format!("Borrowed Pumas helper termination: {e}"))?;
            child
                .try_wait()
                .map_err(|e| e.to_string())?
                .ok_or("Borrowed Pumas helper exit missing")?
        };
        owner.child = None;
        if !exit.success() && service_owned {
            owner.terminal_error = Some(format!("Owned Pumas exited with {exit}"));
        }
    }
    finish_stdout(&mut owner).await
}

async fn finish_stdout(owner: &mut RuntimeOwner) -> Result<(), String> {
    if let Some(handle) = owner.stdout.as_mut() {
        let result = tokio::time::timeout(Duration::from_secs(30), handle)
            .await
            .map_err(|_| "Pumas stdout drain timed out; completion retained for retry")?;
        owner.stdout = None;
        let failure = match result {
            Ok(Ok(_)) => None,
            Ok(Err(error)) => Some(format!("Pumas stdout drain failed: {error}")),
            Err(error) => Some(format!("Pumas stdout drain task failed: {error}")),
        };
        if let Some(failure) = failure {
            owner.terminal_error = Some(match owner.terminal_error.take() {
                Some(previous) => format!("{previous}; {failure}"),
                None => failure,
            });
        }
    }
    match &owner.terminal_error {
        Some(error) => Err(error.clone()),
        None => Ok(()),
    }
}

async fn terminate_startup(child: &mut Child) -> Result<(), String> {
    if let Some(pid) = child.id() {
        // The unreaped child handle retains identity; this is only our bootstrap
        // subprocess. SIGTERM asks Pumas to drain, including before readiness.
        #[cfg(unix)]
        Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()
            .await
            .map_err(|e| e.to_string())?;
        #[cfg(not(unix))]
        {
            let _ = pid;
            child.start_kill().map_err(|e| e.to_string())?;
        }
    }
    tokio::time::timeout(Duration::from_secs(30), child.wait())
        .await
        .map_err(|_| "Pumas cancelled startup drain timed out")?
        .map_err(|e| e.to_string())?;
    Ok(())
}

impl Drop for PumasRuntime {
    fn drop(&mut self) {
        if self
            .owner
            .try_lock()
            .is_ok_and(|owner| owner.child.is_none() && owner.stdout.is_none())
        {
            return;
        }
        let owner = self.owner.clone();
        let description = self.description.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Err(error) = shutdown_owner(&owner, &description).await {
                    tracing::error!(%error,"Owned Pumas drop cleanup failed");
                }
            });
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use crate::pumas_inference::tests::{Fixture, description};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn borrowed_shutdown_has_no_owner_effect_and_owned_shutdown_drains_only_its_child() {
        let fixture = Fixture::start().await;
        let description: HttpServiceDescription =
            serde_json::from_value(description(&fixture.url)).unwrap();
        let borrowed = PumasRuntime {
            description: description.clone(),
            owner: Arc::new(Mutex::new(RuntimeOwner::default())),
        };
        borrowed.shutdown().await.unwrap();
        assert!(fixture.requests.lock().is_empty());
        let mut child = Command::new("sh")
            .args(["-c", "read line"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let requests = fixture.requests.clone();
        let observer = tokio::spawn(async move {
            loop {
                if requests
                    .lock()
                    .iter()
                    .any(|(_, body)| body["method"] == "shutdown")
                {
                    stdin.write_all(b"done\n").await.unwrap();
                    return;
                }
                tokio::task::yield_now().await;
            }
        });
        let owned = PumasRuntime {
            description,
            owner: Arc::new(Mutex::new(RuntimeOwner {
                child: Some(child),
                service_owned: true,
                ..RuntimeOwner::default()
            })),
        };
        owned.shutdown().await.unwrap();
        observer.await.unwrap();
        assert!(owned.owner.lock().await.child.is_none());
        owned.shutdown().await.unwrap();
        assert_eq!(
            fixture
                .requests
                .lock()
                .iter()
                .filter(|(_, body)| body["method"] == "shutdown")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn cancelled_shutdown_retains_stdout_completion_for_the_next_waiter() {
        let fixture = Fixture::start().await;
        let (complete, receipt) = tokio::sync::oneshot::channel();
        let runtime = PumasRuntime {
            description: serde_json::from_value(description(&fixture.url)).unwrap(),
            owner: Arc::new(Mutex::new(RuntimeOwner {
                stdout: Some(tokio::spawn(async move {
                    receipt.await.expect("output completion released");
                    Ok(7)
                })),
                ..RuntimeOwner::default()
            })),
        };
        assert!(
            tokio::time::timeout(Duration::from_millis(20), runtime.shutdown())
                .await
                .is_err()
        );
        assert!(
            runtime.owner.lock().await.stdout.is_some(),
            "cancelled waiter discarded stdout ownership"
        );
        complete.send(()).unwrap();
        runtime.shutdown().await.unwrap();
        assert!(runtime.owner.lock().await.stdout.is_none());
        assert!(fixture.requests.lock().is_empty());
    }

    #[tokio::test]
    async fn stdout_read_error_is_observed_and_remains_the_terminal_shutdown_result() {
        struct FailedOutput;
        impl tokio::io::AsyncRead for FailedOutput {
            fn poll_read(
                self: std::pin::Pin<&mut Self>,
                _context: &mut std::task::Context<'_>,
                _buffer: &mut tokio::io::ReadBuf<'_>,
            ) -> std::task::Poll<std::io::Result<()>> {
                std::task::Poll::Ready(Err(std::io::Error::other("controlled stdout read failure")))
            }
        }
        let fixture = Fixture::start().await;
        let runtime = PumasRuntime {
            description: serde_json::from_value(description(&fixture.url)).unwrap(),
            owner: Arc::new(Mutex::new(RuntimeOwner {
                stdout: Some(tokio::spawn(async {
                    tokio::io::copy(&mut FailedOutput, &mut tokio::io::sink()).await
                })),
                ..RuntimeOwner::default()
            })),
        };
        let error = runtime.shutdown().await.unwrap_err();
        assert!(
            error.contains("Pumas stdout drain failed: controlled stdout read failure"),
            "{error}"
        );
        assert!(runtime.owner.lock().await.stdout.is_none());
        assert_eq!(runtime.shutdown().await.unwrap_err(), error);
        assert!(fixture.requests.lock().is_empty());
    }

    #[tokio::test]
    async fn stdout_task_panic_is_observed_and_not_reported_as_clean_shutdown() {
        let fixture = Fixture::start().await;
        let runtime = PumasRuntime {
            description: serde_json::from_value(description(&fixture.url)).unwrap(),
            owner: Arc::new(Mutex::new(RuntimeOwner {
                stdout: Some(tokio::spawn(async {
                    panic!("controlled stdout task panic")
                })),
                ..RuntimeOwner::default()
            })),
        };
        let error = runtime.shutdown().await.unwrap_err();
        assert!(
            error.contains("Pumas stdout drain task failed")
                && error.contains("controlled stdout task panic"),
            "{error}"
        );
        assert_eq!(runtime.shutdown().await.unwrap_err(), error);
        assert!(runtime.owner.lock().await.stdout.is_none());
    }

    #[tokio::test]
    async fn borrowed_bootstrap_timeout_reaps_only_the_spawned_helper() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::start().await;
        let root = std::env::temp_dir().join(format!(
            "eidetic-borrowed-bootstrap-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let binary = root.join("fixture-pumas");
        let ack = json!({"bootstrap_schema_version":1,"ownership":"borrowed","description":description(&fixture.url)});
        std::fs::write(&binary, format!("#!/bin/sh\nprintf '%s' \"$$\" > '{}/pid'\nprintf '%s\\n' 'PUMAS_LOCAL_ACCESS={ack}'\nexec sleep 300\n", root.display())).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let result = PumasRuntime::start(&root, &binary).await;
        assert!(
            matches!(&result, Err(error) if error.contains("Borrowed Pumas bootstrap did not exit")),
            "unexpected bootstrap result"
        );
        let pid = std::fs::read_to_string(root.join("pid")).unwrap();
        let live_path = format!("/proc/{}", pid.trim());
        let still_alive = Path::new(&live_path).exists();
        // Preserve the original failure without leaving the deliberately hung
        // fixture behind when this regression runs against the old code.
        if still_alive {
            Command::new("kill")
                .args(["-KILL", pid.trim()])
                .status()
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                while Path::new(&live_path).exists() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        }
        assert!(
            fixture.requests.lock().is_empty(),
            "borrowed service received an RPC"
        );
        PumasClient::connect(&fixture.url).await.unwrap();
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            !still_alive,
            "timed-out borrowed bootstrap helper lost its exit observer"
        );
    }

    #[tokio::test]
    async fn cancelled_admitted_startup_still_observes_ack_and_drains_owned_child() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::start().await;
        let root =
            std::env::temp_dir().join(format!("eidetic-pumas-bootstrap-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let binary = root.join("fixture-pumas");
        let ack = json!({"bootstrap_schema_version":1,"ownership":"owned","description":description(&fixture.url)});
        // Delay acknowledgement to deterministically cancel the awaiting caller.
        // The fixture process exits only after the exact fenced shutdown request.
        std::fs::write(&binary,format!("#!/bin/sh\nprintf '%s' \"$$\" > '{}/started'\nsleep 0.1\nprintf '%s\\n' 'PUMAS_LOCAL_ACCESS={ack}'\nwhile [ ! -e '{}/exit' ]; do sleep 0.01; done\n",root.display(),root.display())).unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let requests = fixture.requests.clone();
        let exit_root = root.clone();
        let observer = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if requests
                        .lock()
                        .iter()
                        .any(|(_, body)| body["method"] == "shutdown")
                    {
                        std::fs::write(exit_root.join("exit"), "").unwrap();
                        return;
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        });
        let task_root = root.clone();
        let task =
            tokio::spawn(async move { PumasRuntime::attach(&task_root, Some(&binary)).await });
        tokio::time::timeout(Duration::from_secs(10), async {
            while !root.join("started").exists() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        task.abort();
        assert!(matches!(task.await, Err(error) if error.is_cancelled()));
        observer.await.unwrap();
        // Observe actual child reaping before removing the signal files or ending
        // the test runtime; a shutdown request alone is not cessation evidence.
        let pid = std::fs::read_to_string(root.join("started"))
            .unwrap()
            .parse::<u32>()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            while Path::new(&format!("/proc/{pid}")).exists() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
