use uuid::Uuid;

use crate::{
    db::{migrations::now_utc, projects_repository, Database},
    domain::tasks::{Project, ProjectInput},
    error::AppError,
};

pub async fn list(database: &Database) -> Result<Vec<Project>, AppError> {
    database
        .with_connection(|connection| projects_repository::list(connection))
        .await
}

pub async fn create(database: &Database, input: ProjectInput) -> Result<Project, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let timestamp = now_utc();
            let project = Project {
                id: Uuid::new_v4().to_string(),
                name: input.name,
                color: input.color,
                archived: false,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            };
            projects_repository::insert(connection, &project)?;
            Ok(project)
        })
        .await
}

pub async fn update(
    database: &Database,
    id: String,
    input: ProjectInput,
) -> Result<Project, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let Some(existing) = projects_repository::get(connection, &id)? else {
                return Err(not_found());
            };
            let project = Project {
                id,
                name: input.name,
                color: input.color,
                archived: existing.archived,
                created_at: existing.created_at,
                updated_at: now_utc(),
            };
            projects_repository::update(connection, &project)?;
            Ok(project)
        })
        .await
}

pub async fn set_archived(
    database: &Database,
    id: String,
    archived: bool,
) -> Result<Project, AppError> {
    database
        .with_connection(move |connection| {
            let Some(mut project) = projects_repository::get(connection, &id)? else {
                return Err(not_found());
            };
            project.archived = archived;
            project.updated_at = now_utc();
            projects_repository::update(connection, &project)?;
            Ok(project)
        })
        .await
}

pub async fn delete(database: &Database, id: String) -> Result<(), AppError> {
    database
        .with_connection(move |connection| {
            if projects_repository::delete(connection, &id)? {
                Ok(())
            } else {
                Err(not_found())
            }
        })
        .await
}

fn not_found() -> AppError {
    AppError::new("PROJECT_NOT_FOUND", "The project no longer exists.")
}
