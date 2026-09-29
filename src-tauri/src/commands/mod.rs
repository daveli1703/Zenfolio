use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    db::DataEnvironment,
    domain::settings::{AppSettings, UpdateSettingsInput},
    error::AppError,
    services::{backup, settings},
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
    use crate::db::DataEnvironment;

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
}
