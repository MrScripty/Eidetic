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

    pub fn abort_all(&self) {
        let tasks = std::mem::take(&mut *self.tasks.lock());
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
    async fn supervisor_tracks_and_aborts_spawned_tasks() {
        let supervisor = BackendTaskSupervisor::default();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        supervisor.spawn("test-wait", async move {
            let _ = rx.await;
        });

        assert_eq!(supervisor.active_task_count(), 1);
        supervisor.abort_all();
        drop(tx);
        tokio::task::yield_now().await;

        assert_eq!(supervisor.active_task_count(), 0);
    }
}
