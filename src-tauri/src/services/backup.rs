use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    data_management::backup::create_validated_backup,
    db::{migrations::validate_database_file, Database},
    error::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub path: String,
    pub schema_version: i64,
}

pub async fn create_manual(
    database: &Database,
    destination: PathBuf,
) -> Result<BackupResult, AppError> {
    if destination == database.paths().database {
        return Err(AppError::new(
            "VALIDATION_ERROR",
            "The live database cannot be used as a backup destination.",
        ));
    }
    let backup_path = destination.clone();
    database
        .with_connection(move |connection| {
            create_validated_backup(connection, &backup_path, true)?;
            let status = validate_database_file(&backup_path, true)?;
            Ok(BackupResult {
                path: display_path(&backup_path),
                schema_version: status.current_version,
            })
        })
        .await
}

pub async fn validate(path: PathBuf) -> Result<BackupResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let status = validate_database_file(&path, true).map_err(|error| {
            if matches!(error.code.as_str(), "DATABASE_CORRUPT" | "DATABASE_FOREIGN") {
                AppError::new(
                    "BACKUP_INVALID",
                    "The selected file is not a valid Zenfolio backup.",
                )
            } else {
                error
            }
        })?;
        Ok(BackupResult {
            path: display_path(&path),
            schema_version: status.current_version,
        })
    })
    .await
    .map_err(|_| AppError::internal())?
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
