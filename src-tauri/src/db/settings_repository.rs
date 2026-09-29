use rusqlite::{params, Connection};

use crate::{
    db::migrations::now_utc,
    domain::settings::{
        AppSettings, UpdateSettingsInput, DEFAULT_CURRENCY_CODE, DEFAULT_CURRENCY_EXPONENT,
        DEFAULT_DATE_FORMAT, DEFAULT_THEME, DEFAULT_TIME_FORMAT, MONDAY,
    },
    error::AppError,
};

pub fn ensure_defaults(connection: &Connection, timezone: &str) -> Result<(), AppError> {
    let timestamp = now_utc();
    connection
        .execute(
            "INSERT OR IGNORE INTO app_settings (
                id, currency_code, currency_exponent, application_timezone,
                date_format, time_format, first_weekday, theme, created_at, updated_at
             ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                DEFAULT_CURRENCY_CODE,
                DEFAULT_CURRENCY_EXPONENT,
                timezone,
                DEFAULT_DATE_FORMAT,
                DEFAULT_TIME_FORMAT,
                MONDAY,
                DEFAULT_THEME,
                timestamp,
            ],
        )
        .map_err(AppError::from)?;
    Ok(())
}

pub fn get(connection: &Connection) -> Result<AppSettings, AppError> {
    connection
        .query_row(
            "SELECT currency_code, currency_exponent, application_timezone,
                    date_format, time_format, first_weekday, theme, created_at, updated_at
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    currency_code: row.get(0)?,
                    currency_exponent: row.get(1)?,
                    application_timezone: row.get(2)?,
                    date_format: row.get(3)?,
                    time_format: row.get(4)?,
                    first_weekday: row.get(5)?,
                    theme: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(AppError::from)
}

pub fn update(
    connection: &mut Connection,
    input: &UpdateSettingsInput,
) -> Result<AppSettings, AppError> {
    input.validate()?;
    let transaction = connection.transaction().map_err(AppError::from)?;
    let updated = transaction
        .execute(
            "UPDATE app_settings SET
                currency_code = ?1,
                currency_exponent = ?2,
                application_timezone = ?3,
                date_format = ?4,
                time_format = ?5,
                first_weekday = ?6,
                theme = ?7,
                updated_at = ?8
             WHERE id = 1",
            params![
                input.currency_code,
                input.currency_exponent,
                input.application_timezone,
                input.date_format,
                input.time_format,
                input.first_weekday,
                input.theme,
                now_utc(),
            ],
        )
        .map_err(AppError::from)?;
    if updated != 1 {
        return Err(AppError::database_unavailable());
    }
    transaction.commit().map_err(AppError::from)?;
    get(connection)
}
