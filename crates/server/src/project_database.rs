use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;

/// Owns active project database state for command and projection routes.
///
/// Connections remain short-lived and are opened by blocking route work. This
/// owner centralizes the active database boundary while transitional project
/// mirror/autosave state is retired.
#[derive(Clone)]
pub struct ProjectDatabase {
    active_path: Arc<Mutex<Option<PathBuf>>>,
}

impl ProjectDatabase {
    pub fn new(active_path: Arc<Mutex<Option<PathBuf>>>) -> Self {
        Self { active_path }
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.active_path.lock().clone()
    }

    pub fn set_active_path(&self, path: PathBuf) {
        *self.active_path.lock() = Some(path);
    }

    pub fn open_active_write_connection(&self) -> Result<Connection, ProjectDatabaseError> {
        let path = self
            .active_path()
            .ok_or(ProjectDatabaseError::NoActiveProject)?;
        crate::sqlite::open_write_connection(&path).map_err(ProjectDatabaseError::Sqlite)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectDatabaseError {
    #[error("no project loaded")]
    NoActiveProject,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_path_is_shared_with_transitional_state() {
        let project_path = Arc::new(Mutex::new(None));
        let database = ProjectDatabase::new(project_path.clone());
        let path = PathBuf::from("/tmp/eidetic-test.db");

        database.set_active_path(path.clone());

        assert_eq!(*project_path.lock(), Some(path.clone()));
        assert_eq!(database.active_path(), Some(path));
    }

    #[test]
    fn missing_active_path_rejects_connection_open() {
        let database = ProjectDatabase::new(Arc::new(Mutex::new(None)));

        let error = database.open_active_write_connection().unwrap_err();

        assert!(matches!(error, ProjectDatabaseError::NoActiveProject));
    }
}
