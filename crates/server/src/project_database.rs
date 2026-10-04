use std::path::{Path, PathBuf};
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

    /// Resolve the stored spelling through the same containment boundary as a
    /// requested path before comparing lifecycle identities. This is a fresh
    /// resolution, not a cached alias or an unvalidated display-path comparison.
    pub(crate) fn active_path_identity(
        &self,
        root: &Path,
    ) -> Result<Option<PathBuf>, crate::backend_error::BackendError> {
        self.active_path()
            .map(|path| {
                crate::validation::validate_project_path(path.to_string_lossy().as_ref(), root)
            })
            .transpose()
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

#[cfg(test)]
mod identity_tests {
    use super::*;
    use std::fs;

    fn fixture() -> (ProjectDatabase, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("eidetic-project-identity-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        (ProjectDatabase::new(Arc::new(Mutex::new(None))), root)
    }

    #[test]
    fn absent_active_path_has_no_identity() {
        let (database, root) = fixture();
        assert_eq!(database.active_path_identity(&root).unwrap(), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_and_new_active_paths_resolve_to_the_checked_root() {
        let (database, root) = fixture();
        fs::write(root.join("existing.db"), []).unwrap();
        for file in ["existing.db", "nested/new.db"] {
            database.set_active_path(root.join(file));
            assert_eq!(
                database.active_path_identity(&root).unwrap(),
                Some(root.canonicalize().unwrap().join(file))
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_outside_active_path_is_rejected() {
        let (database, root) = fixture();
        let outside = root.with_extension("outside.db");
        fs::write(&outside, []).unwrap();
        database.set_active_path(outside.clone());
        assert!(database.active_path_identity(&root).is_err());
        fs::remove_file(outside).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn active_symlink_spelling_and_canonical_destination_have_one_identity() {
        let (database, root) = fixture();
        let path = root.join("project.db");
        fs::write(&path, []).unwrap();
        let alias = root.join("alias.db");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        database.set_active_path(alias.clone());
        assert_ne!(
            database.active_path().unwrap(),
            path.canonicalize().unwrap()
        );
        assert_eq!(
            database.active_path_identity(&root).unwrap(),
            Some(path.canonicalize().unwrap())
        );
        // Resolving identity must not publish an owner/session change itself.
        assert_eq!(database.active_path(), Some(alias));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn active_normal_verbatim_and_short_spellings_have_one_identity() {
        let (database, root) = fixture();
        let path = root.join("project.db");
        fs::write(&path, []).unwrap();
        let expected = path.canonicalize().unwrap();
        for spelling in crate::validation::tests::windows_path_spellings(&path) {
            database.set_active_path(spelling);
            assert_eq!(
                database.active_path_identity(&root).unwrap(),
                Some(expected.clone())
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
}
