use crate::schema::Project;
use sqlx::SqlitePool;

pub async fn create_project(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    org_id: &str,
    created_by: &str,
    description: Option<&str>,
) -> Result<Project, sqlx::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, name, description, org_id, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(org_id)
    .bind(created_by)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
}

pub async fn list_projects_by_org(
    pool: &SqlitePool,
    org_id: &str,
) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE org_id = ? ORDER BY updated_at DESC")
        .bind(org_id)
        .fetch_all(pool)
        .await
}
