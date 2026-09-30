use std::path::Path;

use chrono::{SecondsFormat, Utc};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub const ZENFOLIO_APPLICATION_ID: i64 = 0x5A45_4E46;

#[derive(Clone, Copy)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "settings",
        sql: include_str!("../../migrations/0001_settings.sql"),
    },
    Migration {
        version: 2,
        name: "tasks",
        sql: include_str!("../../migrations/0002_tasks.sql"),
    },
    Migration {
        version: 3,
        name: "habits",
        sql: include_str!("../../migrations/0003_habits.sql"),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaStatus {
    pub current_version: i64,
    pub latest_version: i64,
}

pub fn latest_version() -> i64 {
    MIGRATIONS.last().map_or(0, |migration| migration.version)
}

pub fn checksum(sql: &str) -> String {
    Sha256::digest(sql.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn inspect_schema(connection: &Connection) -> Result<SchemaStatus, AppError> {
    let application_id: i64 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(map_inspection_error)?;
    if application_id != ZENFOLIO_APPLICATION_ID {
        return Err(AppError::new(
            "DATABASE_FOREIGN",
            "The existing file is not a Zenfolio database.",
        ));
    }

    verify_integrity(connection)?;

    let has_migration_table = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(map_inspection_error)?
        .is_some();

    if !has_migration_table {
        return Ok(SchemaStatus {
            current_version: 0,
            latest_version: latest_version(),
        });
    }

    let mut statement = connection
        .prepare("SELECT version, name, checksum FROM schema_migrations ORDER BY version ASC")
        .map_err(map_inspection_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(map_inspection_error)?;

    let mut current_version = 0;
    for row in rows {
        let (version, name, stored_checksum) = row.map_err(map_inspection_error)?;
        if version != current_version + 1 {
            return Err(AppError::new(
                "DATABASE_INCOMPATIBLE",
                "The database migration history is incomplete.",
            ));
        }
        let Some(migration) = MIGRATIONS
            .iter()
            .find(|migration| migration.version == version)
        else {
            return Err(AppError::new(
                "DATABASE_INCOMPATIBLE",
                "The database was created by a newer version of Zenfolio.",
            ));
        };
        if migration.name != name || checksum(migration.sql) != stored_checksum {
            return Err(AppError::new(
                "DATABASE_INCOMPATIBLE",
                "The database migration history does not match this Zenfolio build.",
            ));
        }
        current_version = version;
    }

    Ok(SchemaStatus {
        current_version,
        latest_version: latest_version(),
    })
}

pub fn initialize_new_database(connection: &Connection) -> Result<(), AppError> {
    connection
        .pragma_update(None, "application_id", ZENFOLIO_APPLICATION_ID)
        .map_err(AppError::from)
}

pub fn apply_pending(connection: &mut Connection, current_version: i64) -> Result<(), AppError> {
    apply_pending_from(connection, current_version, MIGRATIONS)
}

fn apply_pending_from(
    connection: &mut Connection,
    current_version: i64,
    migrations: &[Migration],
) -> Result<(), AppError> {
    for migration in migrations
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(AppError::from)?;
        transaction
            .execute_batch(migration.sql)
            .map_err(|_| migration_failure())?;
        transaction
            .execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at)
                 VALUES (?1, ?2, ?3, ?4)",
                (
                    migration.version,
                    migration.name,
                    checksum(migration.sql),
                    now_utc(),
                ),
            )
            .map_err(|_| migration_failure())?;
        transaction.commit().map_err(|_| migration_failure())?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn apply_pending_from_for_test(
    connection: &mut Connection,
    current_version: i64,
    migration_count: usize,
) -> Result<(), AppError> {
    apply_pending_from(connection, current_version, &MIGRATIONS[..migration_count])
}

pub fn verify_integrity(connection: &Connection) -> Result<(), AppError> {
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(map_inspection_error)?;
    if integrity != "ok" {
        return Err(AppError::new(
            "DATABASE_CORRUPT",
            "The existing database failed its integrity check.",
        ));
    }

    let foreign_key_failure = connection
        .query_row("PRAGMA foreign_key_check", [], |_| Ok(()))
        .optional()
        .map_err(map_inspection_error)?
        .is_some();
    if foreign_key_failure {
        return Err(AppError::new(
            "DATABASE_CORRUPT",
            "The existing database contains invalid relationships.",
        ));
    }
    Ok(())
}

pub fn validate_database_file(path: &Path, require_latest: bool) -> Result<SchemaStatus, AppError> {
    let connection = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(map_inspection_error)?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(map_inspection_error)?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(map_inspection_error)?;
    let status = inspect_schema(&connection)?;
    if require_latest && status.current_version != status.latest_version {
        return Err(AppError::new(
            "DATABASE_INCOMPATIBLE",
            "The backup schema does not match this Zenfolio version.",
        ));
    }
    Ok(status)
}

pub fn now_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn migration_failure() -> AppError {
    AppError::new(
        "MIGRATION_FAILED",
        "Zenfolio could not upgrade the database. The existing data was preserved.",
    )
}

fn map_inspection_error(error: rusqlite::Error) -> AppError {
    let mapped = AppError::from(error);
    if mapped.code == "WRITE_FAILED" || mapped.code == "INTERNAL_ERROR" {
        AppError::new(
            "DATABASE_CORRUPT",
            "The existing database could not be opened safely.",
        )
    } else {
        mapped
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        apply_pending, apply_pending_from, checksum, initialize_new_database, inspect_schema,
        Migration, MIGRATIONS,
    };

    #[test]
    fn applies_and_verifies_the_migration_manifest() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending(&mut connection, 0).expect("apply migrations");

        let status = inspect_schema(&connection).expect("inspect schema");
        assert_eq!(status.current_version, 3);
        assert_eq!(status.current_version, status.latest_version);
    }

    #[test]
    fn detects_a_changed_applied_checksum() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending(&mut connection, 0).expect("apply migrations");
        connection
            .execute(
                "UPDATE schema_migrations SET checksum = ?1 WHERE version = 1",
                [checksum("changed")],
            )
            .expect("change checksum");

        let error = inspect_schema(&connection).expect_err("reject checksum drift");
        assert_eq!(error.code, "DATABASE_INCOMPATIBLE");
        assert_eq!(MIGRATIONS.len(), 3);
    }

    #[test]
    fn rolls_back_a_failed_migration_and_its_history_record() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending_from(&mut connection, 0, &MIGRATIONS[..1]).expect("base migration");
        let failing = [Migration {
            version: 2,
            name: "failing",
            sql: "CREATE TABLE should_roll_back (id INTEGER); INVALID SQL;",
        }];

        let error = apply_pending_from(&mut connection, 1, &failing).expect_err("migration fails");

        assert_eq!(error.code, "MIGRATION_FAILED");
        let table_count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='should_roll_back'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let version_count: i64 = connection
            .query_row(
                "SELECT count(*) FROM schema_migrations WHERE version=2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 0);
        assert_eq!(version_count, 0);
    }

    #[test]
    fn rejects_a_database_newer_than_the_application() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending(&mut connection, 0).expect("base migration");
        connection
            .execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at)
                 VALUES (4, 'future', ?1, '2026-09-29T00:00:00.000Z')",
                ["0".repeat(64)],
            )
            .unwrap();

        let error = inspect_schema(&connection).expect_err("reject newer database");
        assert_eq!(error.code, "DATABASE_INCOMPATIBLE");
    }

    #[test]
    fn upgrades_a_milestone_three_database_to_tasks() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending_from(&mut connection, 0, &MIGRATIONS[..1]).expect("settings migration");
        assert_eq!(inspect_schema(&connection).unwrap().current_version, 1);

        apply_pending_from(&mut connection, 1, &MIGRATIONS[1..2]).expect("tasks migration");

        assert_eq!(inspect_schema(&connection).unwrap().current_version, 2);
        for table in ["projects", "tags", "tasks", "task_tags"] {
            let count: i64 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing table {table}");
        }
    }

    #[test]
    fn upgrades_a_task_database_to_habits_without_losing_existing_records() {
        let mut connection = Connection::open_in_memory().expect("open database");
        initialize_new_database(&connection).expect("set identity");
        apply_pending_from(&mut connection, 0, &MIGRATIONS[..2]).expect("task schema");
        connection.execute(
            "INSERT INTO projects (id, name, archived, created_at, updated_at) VALUES ('p', 'Work', 0, 'now', 'now')",
            [],
        ).unwrap();
        connection.execute(
            "INSERT INTO app_settings (id, currency_code, currency_exponent, application_timezone, date_format, time_format, first_weekday, theme, created_at, updated_at) VALUES (1, 'VND', 0, 'UTC', 'DD/MM/YYYY', '24h', 1, 'system', 'now', 'now')",
            [],
        ).unwrap();

        apply_pending(&mut connection, 2).expect("habit migration");

        assert_eq!(inspect_schema(&connection).unwrap().current_version, 3);
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM projects", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM app_settings", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        for table in ["habits", "habit_rules", "habit_entries"] {
            let count: i64 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing table {table}");
        }
    }
}
