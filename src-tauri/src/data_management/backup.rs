use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::Utc;
use rusqlite::{backup::Backup, Connection, OpenFlags};

use crate::{db::migrations::validate_database_file, error::AppError};

pub fn create_validated_backup(
    source: &Connection,
    destination: &Path,
    require_latest: bool,
) -> Result<PathBuf, AppError> {
    if destination.exists() {
        return Err(AppError::new(
            "BACKUP_DESTINATION_EXISTS",
            "Choose a new filename for the backup.",
        ));
    }
    let parent = destination.parent().ok_or_else(|| {
        AppError::new(
            "BACKUP_FAILED",
            "The selected backup location is not valid.",
        )
    })?;
    fs::create_dir_all(parent).map_err(|_| backup_failure())?;

    let temporary = temporary_path(destination);
    if temporary.exists() {
        return Err(backup_failure());
    }

    let result = (|| {
        let mut output = Connection::open_with_flags(
            &temporary,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| backup_failure())?;
        let backup = Backup::new(source, &mut output).map_err(|_| backup_failure())?;
        backup
            .run_to_completion(100, Duration::from_millis(10), None)
            .map_err(|_| backup_failure())?;
        drop(backup);
        output.close().map_err(|_| backup_failure())?;

        validate_database_file(&temporary, require_latest).map_err(|_| {
            AppError::new(
                "BACKUP_INVALID",
                "The generated backup did not pass validation.",
            )
        })?;
        fs::rename(&temporary, destination).map_err(|_| backup_failure())?;
        Ok(destination.to_path_buf())
    })();

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn timestamped_backup_path(directory: &Path, prefix: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S-%3f");
    directory.join(format!("{prefix}-{timestamp}.sqlite3"))
}

fn temporary_path(destination: &Path) -> PathBuf {
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zenfolio-backup.sqlite3");
    destination.with_file_name(format!(
        ".{filename}.{}.{}.partial",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ))
}

fn backup_failure() -> AppError {
    AppError::new(
        "BACKUP_FAILED",
        "Zenfolio could not create a verified database backup.",
    )
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    use super::create_validated_backup;
    use crate::db::{migrations, settings_repository};

    #[test]
    fn creates_an_independently_validated_backup_with_committed_data() {
        let directory = tempdir().expect("temporary directory");
        let source_path = directory.path().join("source.sqlite3");
        let backup_path = directory.path().join("backup.sqlite3");
        let mut source = Connection::open(&source_path).expect("source database");
        source.pragma_update(None, "journal_mode", "WAL").unwrap();
        migrations::initialize_new_database(&source).unwrap();
        migrations::apply_pending(&mut source, 0).unwrap();
        settings_repository::ensure_defaults(&source, "Asia/Ho_Chi_Minh").unwrap();
        source
            .execute("UPDATE app_settings SET theme='dark' WHERE id=1", [])
            .unwrap();

        create_validated_backup(&source, &backup_path, true).expect("create backup");

        let backup =
            Connection::open_with_flags(&backup_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open backup independently");
        let theme: String = backup
            .query_row("SELECT theme FROM app_settings WHERE id=1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(theme, "dark");
        assert_eq!(
            migrations::inspect_schema(&backup).unwrap().current_version,
            2
        );
    }

    #[test]
    fn does_not_publish_an_invalid_backup() {
        let directory = tempdir().expect("temporary directory");
        let backup_path = directory.path().join("backup.sqlite3");
        let source = Connection::open_in_memory().expect("foreign database");

        let error =
            create_validated_backup(&source, &backup_path, true).expect_err("reject backup");

        assert_eq!(error.code, "BACKUP_INVALID");
        assert!(!backup_path.exists());
    }
}
