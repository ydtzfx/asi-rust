use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn project_root() -> &'static Path {
    PROJECT_ROOT.get_or_init(|| {
        std::env::current_dir()
            .and_then(|p| p.canonicalize())
            .unwrap_or_else(|_| std::env::current_dir().unwrap())
    })
}

pub fn get_project_root() -> &'static Path {
    project_root()
}

pub async fn resolve_safe_path(file_path: &str) -> Result<PathBuf, String> {
    let absolute = project_root().join(file_path);

    // Try to canonicalize (file exists)
    if let Ok(resolved) = tokio::fs::canonicalize(&absolute).await {
        if resolved.starts_with(project_root()) || resolved == *project_root() {
            return Ok(resolved);
        }
        return Err(format!(
            "Access denied: path \"{}\" resolves outside the project directory",
            file_path
        ));
    }

    // File doesn't exist — walk up to find nearest existing ancestor
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
