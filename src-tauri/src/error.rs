use std::collections::BTreeMap;

use rusqlite::ErrorCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<BTreeMap<String, String>>,
}

impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            field_errors: None,
        }
    }

    pub fn validation(field_errors: BTreeMap<String, String>) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_owned(),
            message: "Check the highlighted settings and try again.".to_owned(),
            field_errors: Some(field_errors),
        }
    }

    pub fn database_unavailable() -> Self {
        Self::new(
            "DATABASE_UNAVAILABLE",
            "The Zenfolio database is not available in the current application state.",
        )
    }

    pub fn internal() -> Self {
        Self::new(
            "INTERNAL_ERROR",
            "Zenfolio could not complete the operation.",
        )
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(details, _) = &error {
            return match details.code {
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => Self::new(
                    "DATABASE_BUSY",
                    "The database is busy. Wait a moment and try again.",
                ),
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase => Self::new(
                    "DATABASE_CORRUPT",
                    "The existing database could not be opened safely.",
                ),
                ErrorCode::DiskFull
                | ErrorCode::ReadOnly
                | ErrorCode::SystemIoFailure
                | ErrorCode::CannotOpen
                | ErrorCode::PermissionDenied => {
                    Self::new("WRITE_FAILED", "Zenfolio could not write to local storage.")
                }
                _ => Self::internal(),
            };
        }

        Self::internal()
    }
}

impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::new("WRITE_FAILED", "Zenfolio could not write to local storage.")
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::ffi;

    use super::AppError;

    #[test]
    fn serializes_the_stable_error_contract() {
        let value = serde_json::to_value(AppError::new(
            "DATABASE_BUSY",
            "The database is busy. Wait a moment and try again.",
        ))
        .expect("serialize error");

        assert_eq!(value["code"], "DATABASE_BUSY");
        assert_eq!(
            value["message"],
            "The database is busy. Wait a moment and try again."
        );
        assert!(value.get("fieldErrors").is_none());
    }

    #[test]
    fn maps_busy_and_disk_full_to_stable_codes() {
        let busy = AppError::from(rusqlite::Error::SqliteFailure(
            ffi::Error::new(ffi::SQLITE_BUSY),
            None,
        ));
        let full = AppError::from(rusqlite::Error::SqliteFailure(
            ffi::Error::new(ffi::SQLITE_FULL),
            None,
        ));

        assert_eq!(busy.code, "DATABASE_BUSY");
        assert_eq!(full.code, "WRITE_FAILED");
    }
}
