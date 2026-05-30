use std::future::Future;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle;

#[derive(Clone, Default)]
pub struct BackendTaskSupervisor {
    tasks: Arc<Mutex<Vec<BackendTask>>>,
}

struct BackendTask {
    name: &'static str,
    handle: JoinHandle<()>,
}

impl BackendTaskSupervisor {
    pub fn spawn<F>(&self, name: &'static str, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(async move {
            tracing::debug!("backend task started: {name}");
            future.await;
            tracing::debug!("backend task stopped: {name}");
        });
        self.tasks.lock().push(BackendTask { name, handle });
    }

    pub async fn shutdown_all(&self) {
        let tasks = self.take_tasks();
        for task in tasks {
            tracing::info!("shutting down backend task: {}", task.name);
            task.handle.abort();
            match task.handle.await {
                Ok(()) => {
                    tracing::debug!("backend task completed during shutdown: {}", task.name);
                }
                Err(error) if error.is_cancelled() => {
                    tracing::debug!("backend task cancelled during shutdown: {}", task.name);
                }
                Err(error) => {
                    tracing::error!(
                        "backend task failed during shutdown: {}: {error}",
                        task.name
                    );
                }
            }
        }
    }

    pub fn abort_all(&self) {
        let tasks = self.take_tasks();
        for task in tasks {
            tracing::info!("aborting backend task: {}", task.name);
            task.handle.abort();
        }
    }

    pub fn active_task_count(&self) -> usize {
        let mut tasks = self.tasks.lock();
        tasks.retain(|task| !task.handle.is_finished());
        tasks.len()
    }

    fn take_tasks(&self) -> Vec<BackendTask> {
        std::mem::take(&mut *self.tasks.lock())
    }
}

impl Drop for BackendTaskSupervisor {
    fn drop(&mut self) {
        if Arc::strong_count(&self.tasks) == 1 {
            self.abort_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BackendTaskSupervisor;

    #[tokio::test]
    async fn supervisor_tracks_and_shuts_down_spawned_tasks() {
        let supervisor = BackendTaskSupervisor::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        supervisor.spawn("test-wait", async move {
            let _ = rx.await;
        });

        assert_eq!(supervisor.active_task_count(), 1);
        supervisor.shutdown_all().await;
        drop(tx);
        tokio::task::yield_now().await;

        assert_eq!(supervisor.active_task_count(), 0);
    }

    #[tokio::test]
    async fn supervisor_shutdown_observes_task_panics() {
        let supervisor = BackendTaskSupervisor::default();

        supervisor.spawn("test-panic", async move {
            panic!("intentional backend task panic");
        });

        tokio::task::yield_now().await;
        supervisor.shutdown_all().await;

        assert_eq!(supervisor.active_task_count(), 0);
    }
}
