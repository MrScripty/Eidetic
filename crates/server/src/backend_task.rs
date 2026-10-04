use std::future::Future;
use std::sync::Arc;

use futures::FutureExt;
use parking_lot::Mutex;
use tokio::task::{JoinError, JoinHandle};

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
        let completed = {
            let mut tasks = self.tasks.lock();
            // Normal command admission reclaims completed records. Retain every
            // unfinished handle so cancellation and shutdown still own its work.
            let completed = take_completed_tasks(&mut tasks);
            let handle = tokio::spawn(async move {
                tracing::debug!("backend task started: {name}");
                future.await;
                tracing::debug!("backend task stopped: {name}");
            });
            tasks.push(BackendTask { name, handle });
            completed
        };
        observe_completed_tasks(completed);
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
        let (completed, count) = {
            let mut tasks = self.tasks.lock();
            let completed = take_completed_tasks(&mut tasks);
            (completed, tasks.len())
        };
        observe_completed_tasks(completed);
        count
    }

    /// Passive test observation: does not reap records or drive runtime cleanup.
    #[cfg(test)]
    pub(crate) fn retained_task_count(&self) -> usize {
        self.tasks.lock().len()
    }

    fn take_tasks(&self) -> Vec<BackendTask> {
        std::mem::take(&mut *self.tasks.lock())
    }
}

type CompletedTask = (&'static str, Result<(), JoinError>);

fn take_completed_tasks(tasks: &mut Vec<BackendTask>) -> Vec<CompletedTask> {
    let mut completed = Vec::new();
    tasks.retain_mut(|task| {
        if !task.handle.is_finished() {
            return true;
        }
        match (&mut task.handle).now_or_never() {
            Some(result) => {
                completed.push((task.name, result));
                false
            }
            None => true,
        }
    });
    completed
}

fn observe_completed_tasks(completed: Vec<CompletedTask>) {
    for (name, result) in completed {
        match result {
            Ok(()) => tracing::debug!("reaped completed backend task: {name}"),
            Err(error) if error.is_cancelled() => {
                tracing::debug!("reaped cancelled backend task: {name}");
            }
            Err(error) => tracing::error!("reaped failed backend task: {name}: {error}"),
        }
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

#[cfg(test)]
mod lifecycle_tests {
    use std::io::Write;
    use std::sync::Arc;

    use parking_lot::Mutex;
    use tokio::sync::oneshot;

    use super::BackendTaskSupervisor;

    async fn wait_until_finished(supervisor: &BackendTaskSupervisor, name: &str) {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let finished = supervisor
                    .tasks
                    .lock()
                    .iter()
                    .find(|task| task.name == name)
                    .unwrap()
                    .handle
                    .is_finished();
                if finished {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn repeated_spawn_reclaims_finished_records_without_active_task_counter() {
        let supervisor = BackendTaskSupervisor::default();
        for _ in 0..64 {
            supervisor.spawn("short-operation", async {});
            wait_until_finished(&supervisor, "short-operation").await;
            assert_eq!(supervisor.retained_task_count(), 1);
        }
        supervisor.shutdown_all().await;
        assert_eq!(supervisor.retained_task_count(), 0);
    }

    struct NotifyOnDrop(Option<oneshot::Sender<()>>);
    impl Drop for NotifyOnDrop {
        fn drop(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    #[tokio::test]
    async fn normal_reaping_keeps_running_work_owned_until_shutdown_joins_cleanup() {
        let supervisor = BackendTaskSupervisor::default();
        let (started, running) = oneshot::channel();
        let (cleanup, mut cleaned) = oneshot::channel();
        supervisor.spawn("running-operation", async move {
            let _cleanup = NotifyOnDrop(Some(cleanup));
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        running.await.unwrap();
        for _ in 0..32 {
            supervisor.spawn("short-operation", async {});
            wait_until_finished(&supervisor, "short-operation").await;
            assert_eq!(supervisor.retained_task_count(), 2);
            assert!(matches!(
                cleaned.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
        }
        supervisor.shutdown_all().await;
        // shutdown_all must await running task teardown, not just detach it.
        assert!(matches!(cleaned.try_recv(), Ok(())));
        assert_eq!(supervisor.retained_task_count(), 0);
    }

    #[derive(Clone)]
    struct LogBuffer(Arc<Mutex<Vec<u8>>>);
    impl Write for LogBuffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn normal_reaping_observes_and_reports_completed_panics() {
        let supervisor = BackendTaskSupervisor::default();
        supervisor.spawn("named-failed-operation", async {
            panic!("expected panic proof");
        });
        wait_until_finished(&supervisor, "named-failed-operation").await;
        let buffer = LogBuffer(Arc::new(Mutex::new(Vec::new())));
        let writer = buffer.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::ERROR)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            supervisor.spawn("next-operation", std::future::pending());
        });
        let output = String::from_utf8(buffer.0.lock().clone()).unwrap();
        assert!(
            output.contains("reaped failed backend task: named-failed-operation"),
            "{output}"
        );
        assert!(output.contains("expected panic proof"), "{output}");
        assert_eq!(supervisor.retained_task_count(), 1);
        supervisor.shutdown_all().await;
    }
}
