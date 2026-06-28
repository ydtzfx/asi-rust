use std::process::Command;
use std::time::Duration;

/// Start auto-backup loop. Runs backup every `interval` hours.
pub fn start_auto_backup(db_path: String, interval_hours: u64, keep_days: u32) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval_hours * 3600);
        loop {
            tokio::time::sleep(interval).await;
            match run_backup(&db_path, keep_days) {
                Ok(path) => tracing::info!("Auto-backup completed: {}", path),
                Err(e) => tracing::error!("Auto-backup failed: {}", e),
            }
        }
    });
}

fn run_backup(db_path: &str, keep_days: u32) -> Result<String, String> {
    let timestamp = chrono_now();
    let backup_dir = "backups";
    std::fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;

    let backup_file = format!("{}/asi_{}.db", backup_dir, timestamp);

    // Use sqlite3 backup command (safe for live databases).
    let output = Command::new("sqlite3")
        .arg(db_path)
        .arg(format!(".backup '{}'", backup_file))
        .output()
        .map_err(|e| format!("sqlite3 not found: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Compress.
    Command::new("gzip")
        .arg("-f")
        .arg(&backup_file)
        .output()
        .map_err(|e| format!("gzip failed: {}", e))?;

    let compressed = format!("{}.gz", backup_file);

    // Cleanup old backups.
    cleanup_old(backup_dir, keep_days);

    Ok(compressed)
}

fn cleanup_old(dir: &str, keep_days: u32) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let cutoff = std::time::SystemTime::now()
            - Duration::from_secs(keep_days as u64 * 86400);
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
