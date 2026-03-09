use std::path::{Component, Path, PathBuf};

use crate::error::ApiError;

const MAX_NAME_LENGTH: usize = 64;

pub fn validate_name(name: &str, field_name: &str) -> Result<(), ApiError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request(format!("{field_name} is required")));
    }
    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(ApiError::bad_request(format!(
            "{field_name} must be at most {MAX_NAME_LENGTH} characters"
        )));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '_' || c == '-')
    {
        return Err(ApiError::bad_request(format!(
            "{field_name} contains unsupported characters"
        )));
    }
    Ok(())
}

pub fn validate_project_path(input: &str, root: &Path) -> Result<PathBuf, ApiError> {
    if input.trim().is_empty() {
        return Err(ApiError::bad_request("path is required"));
    }

    let root = canonical_or_lexical(root)
        .map_err(|e| ApiError::internal(format!("failed to resolve project root: {e}")))?;
    let candidate = normalize_path(if Path::new(input).is_absolute() {
        PathBuf::from(input)
    } else {
        root.join(input)
    });

    if !candidate.starts_with(&root) {
        return Err(ApiError::bad_request("path must stay within the project storage root"));
    }

    if !root.exists() {
        let _ = nearest_existing_ancestor(&root).ok_or_else(|| {
            ApiError::bad_request("project storage root has no existing parent directory")
        })?;
        return Ok(candidate);
    }

    let existing_ancestor = nearest_existing_ancestor(&candidate).ok_or_else(|| {
        ApiError::bad_request("path must resolve under an existing project storage directory")
    })?;
    let canonical_ancestor = existing_ancestor.canonicalize().map_err(|e| {
        ApiError::bad_request(format!("failed to resolve project path ancestor: {e}"))
    })?;

    if !canonical_ancestor.starts_with(&root) {
        return Err(ApiError::bad_request(
            "path resolves outside the project storage root",
        ));
    }

    Ok(candidate)
}

pub fn is_allowed_local_origin(origin: &str) -> bool {
    let Ok(uri) = axum::http::Uri::try_from(origin) else {
        return false;
    };

    let Some(scheme) = uri.scheme_str() else {
        return false;
    };
    if scheme != "http" && scheme != "https" {
        return false;
    }

    matches!(uri.host(), Some("127.0.0.1" | "localhost" | "[::1]" | "::1"))
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
    use super::{is_allowed_local_origin, validate_name, validate_project_path};
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
        assert_eq!(err.1, "project name is required");
    }

    #[test]
    fn validate_name_rejects_symbols() {
        let err = validate_name("bad/name", "project name").unwrap_err();
        assert_eq!(err.1, "project name contains unsupported characters");
    }

    #[test]
    fn validate_project_path_accepts_child_path() {
        let root = temp_dir("accepts-child");
        let resolved = validate_project_path("episode/project.db", &root).unwrap();
        assert_eq!(resolved, root.join("episode/project.db"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_project_path_rejects_escape() {
        let root = temp_dir("rejects-escape");
        let err = validate_project_path("../escape.db", &root).unwrap_err();
        assert_eq!(err.1, "path must stay within the project storage root");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn validate_project_path_rejects_symlink_escape() {
        let root = temp_dir("rejects-symlink-root");
        let outside = temp_dir("rejects-symlink-outside");
        let linked = root.join("linked");
        std::os::unix::fs::symlink(&outside, &linked).unwrap();

        let err = validate_project_path(linked.join("project.db").to_str().unwrap(), &root)
            .unwrap_err();
        assert_eq!(err.1, "path resolves outside the project storage root");
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

    #[test]
    fn local_origin_policy_allows_loopback_hosts() {
        assert!(is_allowed_local_origin("http://127.0.0.1:5173"));
        assert!(is_allowed_local_origin("http://localhost:3000"));
        assert!(is_allowed_local_origin("https://localhost:3000"));
    }

    #[test]
    fn local_origin_policy_rejects_non_loopback_hosts() {
        assert!(!is_allowed_local_origin("https://example.com"));
        assert!(!is_allowed_local_origin("file:///tmp/index.html"));
    }
}
