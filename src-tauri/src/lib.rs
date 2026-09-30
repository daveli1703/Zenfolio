mod commands;
mod data_management;
mod db;
mod domain;
mod error;
mod services;

use db::{DataEnvironment, Database, StoragePaths};
use error::AppError;
use tauri::{Manager, Runtime};

pub struct AppState {
    database: Option<Database>,
    startup_error: Option<AppError>,
    paths: StoragePaths,
}

impl AppState {
    fn new(database: Result<Database, AppError>, paths: StoragePaths) -> Self {
        match database {
            Ok(database) => Self {
                database: Some(database),
                startup_error: None,
                paths,
            },
            Err(error) => Self {
                database: None,
                startup_error: Some(error),
                paths,
            },
        }
    }

    fn database(&self) -> Result<&Database, AppError> {
        self.database
            .as_ref()
            .ok_or_else(AppError::database_unavailable)
    }

    fn startup_status(&self) -> commands::StartupStatus {
        commands::StartupStatus {
            state: if self.database.is_some() {
                "ready".to_owned()
            } else {
                "recovery".to_owned()
            },
            database_path: self.paths.database.to_string_lossy().into_owned(),
            environment: self.paths.environment,
            error: self.startup_error.clone(),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let identifier_root = app.path().app_local_data_dir()?;
            let paths = StoragePaths::new(identifier_root, DataEnvironment::current());
            let initialization_paths = paths.clone();
            let database = tauri::async_runtime::block_on(async move {
                tauri::async_runtime::spawn_blocking(move || {
                    Database::initialize(initialization_paths)
                })
                .await
                .unwrap_or_else(|_| Err(AppError::internal()))
            });
            app.manage(AppState::new(database, paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_startup_status,
            commands::get_storage_info,
            commands::get_settings,
            commands::update_settings,
            commands::create_manual_backup,
            commands::validate_backup,
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::set_project_archived,
            commands::delete_project,
            commands::list_tags,
            commands::create_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::list_tasks,
            commands::get_task,
            commands::create_task,
            commands::update_task,
            commands::set_task_status,
            commands::delete_task,
            commands::list_habits,
            commands::get_habit_detail,
            commands::create_habit,
            commands::update_habit_profile,
            commands::schedule_habit_rule_change,
            commands::archive_habit,
            commands::save_habit_entry,
            commands::list_today_habits,
            commands::get_habit_year,
            commands::get_habit_delete_impact,
            commands::delete_habit,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Zenfolio");
}

fn focus_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
