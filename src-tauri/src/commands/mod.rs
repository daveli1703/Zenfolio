use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    db::DataEnvironment,
    domain::{
        settings::{AppSettings, UpdateSettingsInput},
        tasks::{Project, ProjectInput, Tag, TagInput, Task, TaskFilters, TaskInput},
    },
    error::AppError,
    services::{backup, projects, settings, tags, tasks},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub state: String,
    pub database_path: String,
    pub environment: DataEnvironment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub database_path: String,
    pub backup_directory: String,
    pub environment: DataEnvironment,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPathInput {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityIdInput {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchiveInput {
    pub id: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusInput {
    pub id: String,
    pub status: String,
}

#[tauri::command]
pub fn get_startup_status(state: State<'_, AppState>) -> StartupStatus {
    state.startup_status()
}

#[tauri::command]
pub fn get_storage_info(state: State<'_, AppState>) -> Result<StorageInfo, AppError> {
    let database = state.database()?;
    Ok(StorageInfo {
        database_path: database.paths().database.to_string_lossy().into_owned(),
        backup_directory: database.paths().backups.to_string_lossy().into_owned(),
        environment: database.paths().environment,
    })
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    settings::get(state.database()?).await
}

#[tauri::command]
pub async fn update_settings(
    input: UpdateSettingsInput,
    state: State<'_, AppState>,
) -> Result<AppSettings, AppError> {
    settings::update(state.database()?, input).await
}

#[tauri::command]
pub async fn create_manual_backup(
    input: BackupPathInput,
    state: State<'_, AppState>,
) -> Result<backup::BackupResult, AppError> {
    validate_path(&input.path)?;
    backup::create_manual(state.database()?, PathBuf::from(input.path)).await
}

#[tauri::command]
pub async fn validate_backup(input: BackupPathInput) -> Result<backup::BackupResult, AppError> {
    validate_path(&input.path)?;
    backup::validate(PathBuf::from(input.path)).await
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    projects::list(state.database()?).await
}

#[tauri::command]
pub async fn create_project(
    input: ProjectInput,
    state: State<'_, AppState>,
) -> Result<Project, AppError> {
    projects::create(state.database()?, input).await
}

#[tauri::command]
pub async fn update_project(
    id: String,
    input: ProjectInput,
    state: State<'_, AppState>,
) -> Result<Project, AppError> {
    projects::update(state.database()?, id, input).await
}

#[tauri::command]
pub async fn set_project_archived(
    input: ProjectArchiveInput,
    state: State<'_, AppState>,
) -> Result<Project, AppError> {
    projects::set_archived(state.database()?, input.id, input.archived).await
}

#[tauri::command]
pub async fn delete_project(
    input: EntityIdInput,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    projects::delete(state.database()?, input.id).await
}

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, AppError> {
    tags::list(state.database()?).await
}

#[tauri::command]
pub async fn create_tag(input: TagInput, state: State<'_, AppState>) -> Result<Tag, AppError> {
    tags::create(state.database()?, input).await
}

#[tauri::command]
pub async fn update_tag(
    id: String,
    input: TagInput,
    state: State<'_, AppState>,
) -> Result<Tag, AppError> {
    tags::update(state.database()?, id, input).await
}

#[tauri::command]
pub async fn delete_tag(input: EntityIdInput, state: State<'_, AppState>) -> Result<(), AppError> {
    tags::delete(state.database()?, input.id).await
}

#[tauri::command]
pub async fn list_tasks(
    filters: TaskFilters,
    state: State<'_, AppState>,
) -> Result<Vec<Task>, AppError> {
    tasks::list(state.database()?, filters).await
}

#[tauri::command]
pub async fn get_task(id: String, state: State<'_, AppState>) -> Result<Task, AppError> {
    tasks::get(state.database()?, id).await
}

#[tauri::command]
pub async fn create_task(input: TaskInput, state: State<'_, AppState>) -> Result<Task, AppError> {
    tasks::create(state.database()?, input).await
}

#[tauri::command]
pub async fn update_task(
    id: String,
    input: TaskInput,
    state: State<'_, AppState>,
) -> Result<Task, AppError> {
    tasks::update(state.database()?, id, input).await
}

#[tauri::command]
pub async fn set_task_status(
    input: TaskStatusInput,
    state: State<'_, AppState>,
) -> Result<Task, AppError> {
    tasks::set_status(state.database()?, input.id, input.status).await
}

#[tauri::command]
pub async fn delete_task(input: EntityIdInput, state: State<'_, AppState>) -> Result<(), AppError> {
    tasks::delete(state.database()?, input.id).await
}

fn validate_path(path: &str) -> Result<(), AppError> {
    if path.trim().is_empty() {
        Err(AppError::new(
            "VALIDATION_ERROR",
            "Choose a database backup file.",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BackupPathInput, StartupStatus};
    use crate::{
        db::DataEnvironment,
        domain::tasks::{Project, Tag, Task},
    };

    #[test]
    fn startup_status_uses_the_frontend_contract() {
        let status = StartupStatus {
            state: "ready".to_owned(),
            database_path: "C:/data/productivity.sqlite3".to_owned(),
            environment: DataEnvironment::Development,
            error: None,
        };
        let value = serde_json::to_value(status).expect("serialize status");
        assert_eq!(value["databasePath"], "C:/data/productivity.sqlite3");
        assert_eq!(value["environment"], "development");
        assert!(value.get("error").is_none());
    }

    #[test]
    fn backup_path_uses_camel_case_input() {
        let input: BackupPathInput =
            serde_json::from_str(r#"{"path":"backup.sqlite3"}"#).expect("deserialize path");
        assert_eq!(input.path, "backup.sqlite3");
    }

    #[test]
    fn task_project_and_tag_dtos_use_camel_case() {
        let project = Project {
            id: "project".to_owned(),
            name: "Work".to_owned(),
            color: None,
            archived: false,
            created_at: "created".to_owned(),
            updated_at: "updated".to_owned(),
        };
        let tag = Tag {
            id: "tag".to_owned(),
            name: "Focus".to_owned(),
            color: None,
            created_at: "created".to_owned(),
            updated_at: "updated".to_owned(),
        };
        let task = Task {
            id: "task".to_owned(),
            title: "Write".to_owned(),
            notes: None,
            due_date: Some("2026-09-30".to_owned()),
            due_time: None,
            priority: "high".to_owned(),
            status: "todo".to_owned(),
            project: Some(project),
            tags: vec![tag],
            completed_at: None,
            created_at: "created".to_owned(),
            updated_at: "updated".to_owned(),
        };
        let value = serde_json::to_value(task).unwrap();
        assert_eq!(value["dueDate"], "2026-09-30");
        assert!(value.get("completedAt").is_some());
        assert_eq!(value["project"]["name"], "Work");
        assert_eq!(value["tags"][0]["name"], "Focus");
    }
}
