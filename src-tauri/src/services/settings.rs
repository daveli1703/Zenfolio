use crate::{
    db::{settings_repository, Database},
    domain::settings::{AppSettings, UpdateSettingsInput},
    error::AppError,
};

pub async fn get(database: &Database) -> Result<AppSettings, AppError> {
    database
        .with_connection(|connection| settings_repository::get(connection))
        .await
}

pub async fn update(
    database: &Database,
    input: UpdateSettingsInput,
) -> Result<AppSettings, AppError> {
    database
        .with_connection(move |connection| settings_repository::update(connection, &input))
        .await
}
