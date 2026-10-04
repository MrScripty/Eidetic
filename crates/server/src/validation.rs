use std::path::{Component, Path, PathBuf};

use crate::backend_error::BackendError;

const MAX_NAME_LENGTH: usize = 64;

pub fn validate_name(name: &str, field_name: &str) -> Result<(), BackendError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(BackendError::bad_request(format!(
            "{field_name} is required"
        )));
    }
    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(BackendError::bad_request(format!(
            "{field_name} must be at most {MAX_NAME_LENGTH} characters"
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '_' || c == '-')
    {
        return Err(BackendError::bad_request(format!(
            "{field_name} contains unsupported characters"
        )));
    }
    Ok(())
}

pub fn validate_positive_finite_f32(value: f32, field_name: &str) -> Result<(), BackendError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(BackendError::bad_request(format!(
            "{field_name} must be a positive finite number"
        )));
    }
    Ok(())
}

pub fn validate_project_path(input: &str, root: &Path) -> Result<PathBuf, BackendError> {
    if input.trim().is_empty() {
        return Err(BackendError::bad_request("path is required"));
    }

    let root = canonical_or_lexical(root)
        .map_err(|e| BackendError::internal(format!("failed to resolve project root: {e}")))?;
    let candidate = normalize_path(if Path::new(input).is_absolute() {
        PathBuf::from(input)
    } else {
        root.join(input)
    });

    let root_exists = root.exists();
    // Existing absolute paths can spell a canonical root through an alias,
    // such as Windows 8.3 names. The canonical ancestor check below is the
    // authority for those paths. Relative traversal and missing roots retain
    // their lexical boundary; Windows aliases must also stay on one drive/share.
    if !path_is_within(&candidate, &root) && (!Path::new(input).is_absolute() || !root_exists) {
        return Err(BackendError::bad_request(
            "path must stay within the project storage root",
        ));
    }

    #[cfg(windows)]
    if root_exists && !same_storage_volume(&candidate, &root) {
        return Err(BackendError::bad_request(
            "path must stay within the project storage root",
        ));
    }

    if !root_exists {
        let _ = nearest_existing_ancestor(&root).ok_or_else(|| {
            BackendError::bad_request("project storage root has no existing parent directory")
        })?;
        return Ok(candidate);
    }

    let existing_ancestor = nearest_existing_ancestor(&candidate).ok_or_else(|| {
        BackendError::bad_request("path must resolve under an existing project storage directory")
    })?;
    let canonical_ancestor = existing_ancestor.canonicalize().map_err(|e| {
        BackendError::bad_request(format!("failed to resolve project path ancestor: {e}"))
    })?;

    if !path_is_within(&canonical_ancestor, &root) {
        return Err(BackendError::bad_request(
            "path resolves outside the project storage root",
        ));
    }

    Ok(candidate)
}

fn path_is_within(candidate: &Path, root: &Path) -> bool {
    #[cfg(not(windows))]
    {
        candidate.starts_with(root)
    }
    #[cfg(windows)]
    {
        // canonicalize adds a verbatim prefix on Windows. Compare the same drive
        // or UNC share equivalently; every remaining directory component still
        // must match, and the canonical ancestor check still rejects symlinks.
        let mut components = candidate.components();
        root.components()
            .all(|expected| match (components.next(), expected) {
                (Some(Component::Prefix(actual)), Component::Prefix(expected)) => {
                    ordinary_windows_prefix(actual.kind())
                        == ordinary_windows_prefix(expected.kind())
                }
                (Some(actual), expected) => actual == expected,
                (None, _) => false,
            })
    }
}

#[cfg(windows)]
fn ordinary_windows_prefix(prefix: std::path::Prefix<'_>) -> std::path::Prefix<'_> {
    use std::path::Prefix;
    match prefix {
        Prefix::VerbatimDisk(drive) => Prefix::Disk(drive),
        Prefix::VerbatimUNC(server, share) => Prefix::UNC(server, share),
        other => other,
    }
}

#[cfg(windows)]
fn same_storage_volume(candidate: &Path, root: &Path) -> bool {
    candidate.is_absolute()
        && match (candidate.components().next(), root.components().next()) {
            (Some(Component::Prefix(actual)), Some(Component::Prefix(expected))) => {
                ordinary_windows_prefix(actual.kind()) == ordinary_windows_prefix(expected.kind())
            }
            _ => false,
        }
}

fn canonical_or_lexical(path: &Path) -> std::io::Result<PathBuf> {
    if path.exists() {
        path.canonicalize()
    } else if path.is_absolute() {
        Ok(normalize_path(path.to_path_buf()))
    } else {
        Ok(normalize_path(std::env::current_dir()?.join(path)))
    }
}

fn nearest_existing_ancestor(path: &Path) -> Option<&Path> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if candidate.exists() {
            return Some(candidate);
        }
        current = candidate.parent();
    }
    None
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{validate_name, validate_project_path};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eidetic-validation-{label}-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn validate_name_rejects_empty() {
        let err = validate_name("   ", "project name").unwrap_err();
        assert_eq!(err.message(), "project name is required");
    }

    #[test]
    fn validate_name_rejects_symbols() {
        let err = validate_name("bad/name", "project name").unwrap_err();
        assert_eq!(
            err.message(),
            "project name contains unsupported characters"
        );
    }

    #[test]
    fn validate_project_path_accepts_child_path() {
        let root = temp_dir("accepts-child");
        let resolved = validate_project_path("episode/project.db", &root).unwrap();
        let expected = root
            .canonicalize()
            .unwrap()
            .join("episode")
            .join("project.db");
        assert_eq!(resolved, expected);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_project_path_accepts_absolute_existing_and_new_files() {
        let root = temp_dir("absolute-files");
        let existing = root.join("existing.db");
        fs::write(&existing, []).unwrap();
        for (kind, path) in [
            ("existing absolute", existing),
            ("new absolute", root.join("new.db")),
            (
                "new canonical",
                root.canonicalize().unwrap().join("canonical.db"),
            ),
        ] {
            let result = validate_project_path(path.to_str().unwrap(), &root);
            assert!(
                result.is_ok(),
                "case={kind}, candidate={path:?}, supplied_root={root:?}, canonical_root={:?}, \
                 existing_ancestor={:?}, canonical_ancestor={:?}, result={result:?}",
                root.canonicalize(),
                super::nearest_existing_ancestor(&path),
                super::nearest_existing_ancestor(&path).map(std::fs::canonicalize),
            );
        }
        let outside = root
            .with_file_name(format!(
                "{}-sibling",
                root.file_name().unwrap().to_str().unwrap()
            ))
            .join("outside.db");
        assert!(validate_project_path(outside.to_str().unwrap(), &root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_prefix_equivalence_keeps_drive_share_and_component_boundaries() {
        use super::path_is_within;
        use std::path::Path;
        for (candidate, root, accepted) in [
            (r"C:\projects\episode.db", r"\\?\C:\projects", true),
            (r"\\?\C:\projects\episode.db", r"C:\projects", true),
            (r"D:\projects\episode.db", r"\\?\C:\projects", false),
            (r"C:\projects-sibling\episode.db", r"\\?\C:\projects", false),
            (
                r"\\server\share\projects\episode.db",
                r"\\?\UNC\server\share\projects",
                true,
            ),
            (
                r"\\other\share\projects\episode.db",
                r"\\?\UNC\server\share\projects",
                false,
            ),
            (
                r"\\server\other\projects\episode.db",
                r"\\?\UNC\server\share\projects",
                false,
            ),
        ] {
            assert_eq!(
                path_is_within(Path::new(candidate), Path::new(root)),
                accepted
            );
        }
    }

    #[test]
    fn validate_project_path_rejects_escape() {
        let root = temp_dir("rejects-escape");
        let err = validate_project_path("../escape.db", &root).unwrap_err();
        assert_eq!(
            err.message(),
            "path must stay within the project storage root"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    fn short_path(path: &std::path::Path) -> PathBuf {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        #[link(name = "kernel32")]
        unsafe extern "system" {
            #[link_name = "GetShortPathNameW"]
            fn get_short_path_name_w(input: *const u16, output: *mut u16, capacity: u32) -> u32;
        }
        let input: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Both calls use a live, NUL-terminated input. The first requests the
        // required UTF-16 capacity; the second writes only into that allocation.
        let capacity = unsafe { get_short_path_name_w(input.as_ptr(), std::ptr::null_mut(), 0) };
        assert!(
            capacity > 0 && capacity <= 32768,
            "GetShortPathNameW sizing failed: {}",
            std::io::Error::last_os_error()
        );
        let mut output = vec![0_u16; capacity as usize];
        let length =
            unsafe { get_short_path_name_w(input.as_ptr(), output.as_mut_ptr(), capacity) };
        assert!(
            length > 0 && length < capacity,
            "GetShortPathNameW failed: {}",
            std::io::Error::last_os_error()
        );
        output.truncate(length as usize);
        PathBuf::from(std::ffi::OsString::from_wide(&output))
    }

    #[cfg(windows)]
    #[test]
    fn validate_project_path_accepts_windows_short_alias_and_rejects_escape() {
        let root = temp_dir("windows-short-alias");
        let canonical_root = root.canonicalize().unwrap();
        let alias = short_path(&canonical_root);
        assert_eq!(alias.canonicalize().unwrap(), canonical_root);
        assert!(
            alias.to_string_lossy().contains('~'),
            "Windows short-alias fixture requires an actual 8.3 name: {alias:?}"
        );
        assert!(!super::path_is_within(&alias, &canonical_root));
        fs::write(root.join("existing.db"), []).unwrap();
        for storage_root in [&canonical_root, &alias] {
            for path in [
                alias.join("existing.db"),
                alias.join("new.db"),
                alias.join("new-directory/project.db"),
            ] {
                assert_eq!(
                    validate_project_path(path.to_str().unwrap(), storage_root).unwrap(),
                    path
                );
            }
        }
        assert_eq!(
            validate_project_path("relative/project.db", &alias).unwrap(),
            canonical_root.join("relative/project.db")
        );
        for path in [
            alias.join("../escape.db"),
            root.with_file_name(format!(
                "{}-sibling",
                root.file_name().unwrap().to_str().unwrap()
            ))
            .join("project.db"),
        ] {
            assert!(validate_project_path(path.to_str().unwrap(), &canonical_root).is_err());
        }
        let outside = temp_dir("windows-short-alias-outside");
        fs::write(outside.join("existing.db"), []).unwrap();
        let link = root.join("escape");
        std::os::windows::fs::symlink_dir(&outside, &link).unwrap();
        for spelling in [&alias, &canonical_root] {
            for file in ["existing.db", "new-directory/project.db"] {
                let error = validate_project_path(
                    spelling.join("escape").join(file).to_str().unwrap(),
                    &canonical_root,
                )
                .unwrap_err();
                assert_eq!(
                    error.message(),
                    "path resolves outside the project storage root"
                );
            }
        }
        fs::remove_dir(link).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn storage_volume_guard_preserves_absolute_drive_and_share_boundaries() {
        use std::path::Path;
        for (candidate, root, accepted) in [
            (r"C:\alias\file.db", r"\\?\C:\storage", true),
            (r"D:\alias\file.db", r"\\?\C:\storage", false),
            (r"C:relative.db", r"\\?\C:\storage", false),
            (
                r"\\server\share\alias\file.db",
                r"\\?\UNC\server\share\storage",
                true,
            ),
            (
                r"\\other\share\alias\file.db",
                r"\\?\UNC\server\share\storage",
                false,
            ),
            (
                r"\\server\other\alias\file.db",
                r"\\?\UNC\server\share\storage",
                false,
            ),
            (r"C:\alias\file.db", r"\\?\UNC\server\share\storage", false),
        ] {
            assert_eq!(
                super::same_storage_volume(Path::new(candidate), Path::new(root)),
                accepted
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn validate_project_path_accepts_root_alias_and_rejects_link_escape() {
        let parent = temp_dir("root-alias");
        let physical = parent.join("physical");
        fs::create_dir(&physical).unwrap();
        let alias = parent.join("alias");
        std::os::unix::fs::symlink(&physical, &alias).unwrap();
        fs::write(physical.join("existing.db"), []).unwrap();
        for storage_root in [&alias, &physical] {
            for path in [alias.join("existing.db"), alias.join("new.db")] {
                assert_eq!(
                    validate_project_path(path.to_str().unwrap(), storage_root).unwrap(),
                    path
                );
            }
        }
        let canonical = physical.canonicalize().unwrap().join("canonical.db");
        assert_eq!(
            validate_project_path(canonical.to_str().unwrap(), &alias).unwrap(),
            canonical
        );
        assert!(
            validate_project_path(
                parent.join("alias-sibling/project.db").to_str().unwrap(),
                &alias
            )
            .is_err()
        );
        let outside = parent.join("outside");
        fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, physical.join("escape")).unwrap();
        for root in [&alias, &physical] {
            let error =
                validate_project_path(root.join("escape/project.db").to_str().unwrap(), root)
                    .unwrap_err();
            assert_eq!(
                error.message(),
                "path resolves outside the project storage root"
            );
        }
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn validate_project_path_rejects_symlink_escape() {
        let root = temp_dir("rejects-symlink-root");
        let outside = temp_dir("rejects-symlink-outside");
        let linked = root.join("linked");
        std::os::unix::fs::symlink(&outside, &linked).unwrap();

        let err =
            validate_project_path(linked.join("project.db").to_str().unwrap(), &root).unwrap_err();
        assert_eq!(
            err.message(),
            "path resolves outside the project storage root"
        );
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn validate_project_path_requires_existing_root_parent() {
        let root = temp_dir("missing-root");
        fs::remove_dir_all(&root).unwrap();

        let resolved = validate_project_path("episode/project.db", &root).unwrap();
        assert_eq!(resolved, root.join("episode/project.db"));
    }
}
