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
};

pub(crate) struct PumasRuntime {
    pub(crate) description: HttpServiceDescription,
    owner: Arc<Mutex<Option<Child>>>,
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
            owner: Arc::new(Mutex::new(None)),
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
        // Keep stdout draining so the actual owner cannot block on its log pipe.
        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut reader, &mut tokio::io::sink()).await;
        });
        if ack.ownership == "borrowed" {
            let exit = tokio::time::timeout(Duration::from_secs(10), child.wait())
                .await
                .map_err(|_| "Borrowed Pumas bootstrap did not exit")?
                .map_err(|e| e.to_string())?;
            if !exit.success() {
                return Err(format!("Borrowed Pumas bootstrap exited with {exit}"));
            }
        }
        let runtime = Self {
            description: ack.description,
            owner: Arc::new(Mutex::new(if ack.ownership == "owned" {
                Some(child)
            } else {
                None
            })),
        };
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
    owner: &Mutex<Option<Child>>,
    description: &HttpServiceDescription,
) -> Result<(), String> {
    let mut slot = owner.lock().await;
    let Some(child) = slot.as_mut() else {
        return Ok(());
    };
    // An exited child has no shutdown authority over a replacement service.
    if child.try_wait().map_err(|e| e.to_string())?.is_some() {
        *slot = None;
        return Ok(());
    }
    let client = PumasClient::for_description(description.clone())?;
    tokio::time::timeout(Duration::from_secs(30), client.rpc("shutdown", json!({})))
        .await
        .map_err(|_| "Owned Pumas shutdown request timed out")??;
    let exit = tokio::time::timeout(Duration::from_secs(30), child.wait())
        .await
        .map_err(|_| "Owned Pumas drain timed out; child retained for retry")?
        .map_err(|e| e.to_string())?;
    *slot = None;
    if !exit.success() {
        return Err(format!("Owned Pumas exited with {exit}"));
    }
    Ok(())
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
            owner: Arc::new(Mutex::new(None)),
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
            owner: Arc::new(Mutex::new(Some(child))),
        };
        owned.shutdown().await.unwrap();
        observer.await.unwrap();
        assert!(owned.owner.lock().await.is_none());
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
