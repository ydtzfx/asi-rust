use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn project_root() -> &'static Path {
    PROJECT_ROOT.get_or_init(|| {
        std::env::current_dir()
            .and_then(|p| p.canonicalize())
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to canonicalize current_dir, using as-is");
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            })
    })
}

pub fn get_project_root() -> &'static Path {
    project_root()
}

/// Resolve a user-supplied file path safely within the project root.
///
/// For existing files: canonicalizes and verifies containment.
/// For non-existing files: walks up to the nearest existing ancestor,
/// canonicalizes that, and verifies containment.
///
/// **TOCTOU note**: for new files, between this check and the actual write,
/// a parent directory could be replaced with a symlink.  Callers should use
/// [`verify_path_after_write`] after creating the file to re-verify containment.
pub async fn resolve_safe_path(file_path: &str) -> Result<PathBuf, String> {
    let absolute = project_root().join(file_path);

    // Try to canonicalize (file exists).
    if let Ok(resolved) = tokio::fs::canonicalize(&absolute).await {
        if resolved.starts_with(project_root()) || resolved == *project_root() {
            return Ok(resolved);
        }
        return Err(format!(
            "Access denied: path \"{}\" resolves outside the project directory",
            file_path
        ));
    }

    // File doesn't exist — walk up to find nearest existing ancestor.
    let mut current = absolute.parent().map(Path::to_path_buf);
    while let Some(ref dir) = current {
        if let Ok(resolved_parent) = tokio::fs::canonicalize(&dir).await {
            if resolved_parent.starts_with(project_root()) || resolved_parent == *project_root() {
                return Ok(absolute);
            }
            return Err(format!(
                "Access denied: path \"{}\" resolves outside the project directory",
                file_path
            ));
        }
        let parent = dir.parent().map(Path::to_path_buf);
        if parent == current {
            break;
        }
        current = parent;
    }

    Err(format!(
        "Access denied: path \"{}\" resolves outside the project directory",
        file_path
    ))
}

/// Re-verify that a newly created file is still within the project root.
/// Call this after writing a new file to mitigate TOCTOU (symlink swap) attacks.
pub async fn verify_path_after_write(path: &Path) -> Result<(), String> {
    let resolved = tokio::fs::canonicalize(path)
        .await
        .map_err(|e| format!("Failed to canonicalize after write: {}", e))?;
    if resolved.starts_with(project_root()) || resolved == *project_root() {
        Ok(())
    } else {
        // Path escaped the sandbox after write — delete the file.
        let _ = tokio::fs::remove_file(path).await;
        Err("Access denied: path escaped project directory after write".into())
    }
}
