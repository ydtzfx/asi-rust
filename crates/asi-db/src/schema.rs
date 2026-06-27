/// Table and column name constants for compile-time safety.
pub mod tables {
    pub const USERS: &str = "users";
    pub const PROJECTS: &str = "projects";
    pub const SESSIONS: &str = "sessions";
    pub const AUDIT_LOG: &str = "audit_log";
}

/// Struct matching the `users` table row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub image_url: Option<String>,
    pub org_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub context_json: Option<String>,
    pub message_count: i64,
    pub token_used: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: i64,
    pub session_id: Option<String>,
    pub user_id: String,
    pub action: String,
    pub summary: String,
    pub detail_json: Option<String>,
    pub ip: Option<String>,
    pub created_at: i64,
}
