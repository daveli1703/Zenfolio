pub mod goals_repository;
pub mod habits_repository;
pub mod migrations;
pub mod projects_repository;
pub mod settings_repository;
pub mod tags_repository;
pub mod tasks_repository;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::{
    data_management::backup::{create_validated_backup, timestamped_backup_path},
    error::AppError,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataEnvironment {
    Development,
    Production,
    Test,
}

impl DataEnvironment {
    pub fn current() -> Self {
        if cfg!(debug_assertions) {
            Self::Development
        } else {
            Self::Production
        }
    }

    fn directory_name(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Test => "test",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub environment: DataEnvironment,
    pub root: PathBuf,
    pub database: PathBuf,
    pub backups: PathBuf,
    pub migration_backups: PathBuf,
}

impl StoragePaths {
    pub fn new(identifier_root: PathBuf, environment: DataEnvironment) -> Self {
        let root = identifier_root.join(environment.directory_name());
        let backups = root.join("backups");
        Self {
            environment,
            database: root.join("productivity.sqlite3"),
            migration_backups: backups.join("migration"),
            backups,
            root,
        }
    }
}

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Option<Connection>>>,
    paths: StoragePaths,
}

impl Database {
    pub fn initialize(paths: StoragePaths) -> Result<Self, AppError> {
        fs::create_dir_all(&paths.root)?;
        let existed = paths.database.exists();
        let current_version = if existed {
            let status = migrations::validate_database_file(&paths.database, false)?;
            if status.current_version < status.latest_version {
                fs::create_dir_all(&paths.migration_backups).map_err(|_| {
                    AppError::new(
                        "MIGRATION_BACKUP_FAILED",
                        "Zenfolio could not create the required pre-migration backup directory. The database was not changed.",
                    )
                })?;
                let source = open_read_only(&paths.database)?;
                let backup_path = timestamped_backup_path(
                    &paths.migration_backups,
                    &format!("pre-migration-v{}", status.current_version),
                );
                create_validated_backup(&source, &backup_path, false).map_err(|_| {
                    AppError::new(
                        "MIGRATION_BACKUP_FAILED",
                        "Zenfolio could not verify the required pre-migration backup. The database was not changed.",
                    )
                })?;
            }
            status.current_version
        } else {
            0
        };

        let mut connection = open_live(&paths.database, !existed)?;
        if !existed {
            migrations::initialize_new_database(&connection)?;
        }
        migrations::apply_pending(&mut connection, current_version)?;
        let status = migrations::inspect_schema(&connection)?;
        if status.current_version != status.latest_version {
            return Err(AppError::new(
                "DATABASE_INCOMPATIBLE",
                "The database schema is not supported by this Zenfolio version.",
            ));
        }

        let timezone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_owned());
        let timezone = if timezone.parse::<chrono_tz::Tz>().is_ok() {
            timezone
        } else {
            "UTC".to_owned()
        };
        settings_repository::ensure_defaults(&connection, &timezone)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(Some(connection))),
            paths,
        })
    }

    pub fn paths(&self) -> &StoragePaths {
        &self.paths
    }

    pub async fn with_connection<T, F>(&self, operation: F) -> Result<T, AppError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, AppError> + Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        tauri::async_runtime::spawn_blocking(move || {
            let mut guard = connection.lock().map_err(|_| AppError::internal())?;
            let connection = guard.as_mut().ok_or_else(AppError::database_unavailable)?;
            operation(connection)
        })
        .await
        .map_err(|_| AppError::internal())?
    }
}

fn open_live(path: &Path, create: bool) -> Result<Connection, AppError> {
    let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    if create {
        flags |= OpenFlags::SQLITE_OPEN_CREATE;
    }
    let connection = Connection::open_with_flags(path, flags).map_err(AppError::from)?;
    configure_connection(&connection)?;
    Ok(connection)
}

fn open_read_only(path: &Path) -> Result<Connection, AppError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(AppError::from)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
}

fn configure_connection(connection: &Connection) -> Result<(), AppError> {
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    let foreign_keys: i64 =
        connection.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(AppError::new(
            "DATABASE_UNAVAILABLE",
            "Zenfolio could not enable database relationship checks.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::Connection;
    use tempfile::tempdir;

    use super::{DataEnvironment, Database, StoragePaths};
    use crate::{
        db::{migrations, settings_repository},
        domain::settings::UpdateSettingsInput,
    };

    #[test]
    fn separates_all_storage_environments() {
        let base = tempdir().expect("temporary directory");
        let development = StoragePaths::new(base.path().to_owned(), DataEnvironment::Development);
        let production = StoragePaths::new(base.path().to_owned(), DataEnvironment::Production);
        let test = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);

        assert_ne!(development.database, production.database);
        assert_ne!(development.database, test.database);
        assert!(development
            .database
            .ends_with("development/productivity.sqlite3"));
    }

    #[test]
    fn creates_a_configured_database_and_persists_settings() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        let database = Database::initialize(paths.clone()).expect("initialize database");
        {
            let mut guard = database.connection.lock().expect("database lock");
            let connection = guard.as_mut().expect("open connection");
            let journal_mode: String = connection
                .pragma_query_value(None, "journal_mode", |row| row.get(0))
                .expect("journal mode");
            let synchronous: i64 = connection
                .pragma_query_value(None, "synchronous", |row| row.get(0))
                .expect("synchronous");
            let foreign_keys: i64 = connection
                .pragma_query_value(None, "foreign_keys", |row| row.get(0))
                .expect("foreign keys");
            assert_eq!(journal_mode.to_lowercase(), "wal");
            assert_eq!(synchronous, 2);
            assert_eq!(foreign_keys, 1);
            assert_eq!(
                settings_repository::get(connection).unwrap().currency_code,
                "VND"
            );
            settings_repository::update(
                connection,
                &UpdateSettingsInput {
                    currency_code: "VND".to_owned(),
                    currency_exponent: 0,
                    application_timezone: "Asia/Ho_Chi_Minh".to_owned(),
                    date_format: "YYYY-MM-DD".to_owned(),
                    time_format: "24h".to_owned(),
                    first_weekday: 1,
                    theme: "dark".to_owned(),
                },
            )
            .expect("update settings");
        }
        drop(database);

        let reopened = Database::initialize(paths).expect("reopen database");
        let connection = reopened.connection.lock().expect("database lock");
        let connection = connection.as_ref().expect("open connection");
        assert_eq!(
            migrations::inspect_schema(connection)
                .unwrap()
                .current_version,
            4
        );
        assert_eq!(settings_repository::get(connection).unwrap().theme, "dark");
    }

    #[test]
    fn creates_a_validated_backup_before_migrating_an_existing_database() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        fs::create_dir_all(&paths.root).expect("storage root");
        let connection = Connection::open(&paths.database).expect("version zero database");
        migrations::initialize_new_database(&connection).expect("set Zenfolio identity");
        drop(connection);

        let database = Database::initialize(paths.clone()).expect("migrate database");
        drop(database);

        let backups = fs::read_dir(&paths.migration_backups)
            .expect("migration backups")
            .collect::<Result<Vec<_>, _>>()
            .expect("backup entries");
        assert_eq!(backups.len(), 1);
        let status = migrations::validate_database_file(&backups[0].path(), false)
            .expect("validated pre-migration backup");
        assert_eq!(status.current_version, 0);
        assert_eq!(
            migrations::validate_database_file(&paths.database, true)
                .unwrap()
                .current_version,
            4
        );
    }

    #[test]
    fn upgrades_a_milestone_three_file_after_backing_up_schema_one() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        fs::create_dir_all(&paths.root).expect("storage root");
        let mut connection = Connection::open(&paths.database).expect("milestone three database");
        migrations::initialize_new_database(&connection).expect("set Zenfolio identity");
        super::migrations::apply_pending_from_for_test(&mut connection, 0, 1)
            .expect("settings migration");
        drop(connection);

        let database = Database::initialize(paths.clone()).expect("upgrade database");
        drop(database);

        let backups = fs::read_dir(&paths.migration_backups)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            migrations::validate_database_file(&backups[0].path(), false)
                .unwrap()
                .current_version,
            1
        );
        assert_eq!(
            migrations::validate_database_file(&paths.database, true)
                .unwrap()
                .current_version,
            4
        );
    }

    #[test]
    fn task_data_persists_after_database_restart() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        let database = Database::initialize(paths.clone()).expect("initialize database");
        {
            let mut guard = database.connection.lock().unwrap();
            let connection = guard.as_mut().unwrap();
            connection
                .execute(
                    "INSERT INTO tasks (
                        id, title, priority, status, created_at, updated_at
                     ) VALUES ('persisted', 'Persistent task', 'medium', 'todo', 'now', 'now')",
                    [],
                )
                .unwrap();
        }
        drop(database);

        let reopened = Database::initialize(paths).expect("reopen database");
        let guard = reopened.connection.lock().unwrap();
        let title: String = guard
            .as_ref()
            .unwrap()
            .query_row("SELECT title FROM tasks WHERE id='persisted'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(title, "Persistent task");
    }

    #[test]
    fn preserves_an_existing_corrupt_database() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        fs::create_dir_all(&paths.root).expect("storage root");
        let original = b"not a sqlite database";
        fs::write(&paths.database, original).expect("corrupt fixture");

        let error = initialization_error(Database::initialize(paths.clone()));

        assert_eq!(error.code, "DATABASE_CORRUPT");
        assert_eq!(fs::read(paths.database).unwrap(), original);
    }

    #[test]
    fn rejects_a_foreign_sqlite_database_without_replacing_it() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        fs::create_dir_all(&paths.root).expect("storage root");
        let connection = Connection::open(&paths.database).expect("foreign database");
        connection
            .execute_batch("CREATE TABLE foreign_data (value TEXT);")
            .expect("foreign schema");
        drop(connection);
        let before = fs::read(&paths.database).expect("read fixture");

        let error = initialization_error(Database::initialize(paths.clone()));

        assert_eq!(error.code, "DATABASE_FOREIGN");
        assert_eq!(fs::read(paths.database).unwrap(), before);
    }

    #[test]
    fn stops_before_migration_when_the_required_backup_cannot_be_created() {
        let base = tempdir().expect("temporary directory");
        let paths = StoragePaths::new(base.path().to_owned(), DataEnvironment::Test);
        fs::create_dir_all(&paths.root).expect("storage root");
        let connection = Connection::open(&paths.database).expect("version zero database");
        migrations::initialize_new_database(&connection).expect("set Zenfolio identity");
        drop(connection);
        fs::create_dir_all(&paths.backups).expect("backup directory");
        fs::write(&paths.migration_backups, b"blocks directory creation").expect("blocking file");

        let error = initialization_error(Database::initialize(paths.clone()));

        assert_eq!(error.code, "MIGRATION_BACKUP_FAILED");
        let connection = Connection::open(&paths.database).expect("unchanged database");
        let has_migrations: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(has_migrations, 0);
    }

    fn initialization_error(
        result: Result<Database, crate::error::AppError>,
    ) -> crate::error::AppError {
        match result {
            Ok(_) => panic!("database initialization should fail"),
            Err(error) => error,
        }
    }
}
