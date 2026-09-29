use std::collections::BTreeMap;

use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const DEFAULT_DATE_FORMAT: &str = "DD/MM/YYYY";
pub const DEFAULT_TIME_FORMAT: &str = "24h";
pub const DEFAULT_THEME: &str = "system";
pub const DEFAULT_CURRENCY_CODE: &str = "VND";
pub const DEFAULT_CURRENCY_EXPONENT: i64 = 0;
pub const MONDAY: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub currency_code: String,
    pub currency_exponent: i64,
    pub application_timezone: String,
    pub date_format: String,
    pub time_format: String,
    pub first_weekday: i64,
    pub theme: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsInput {
    pub currency_code: String,
    pub currency_exponent: i64,
    pub application_timezone: String,
    pub date_format: String,
    pub time_format: String,
    pub first_weekday: i64,
    pub theme: String,
}

impl UpdateSettingsInput {
    pub fn validate(&self) -> Result<(), AppError> {
        let mut errors = BTreeMap::new();
        let currency_is_valid = self.currency_code.len() == 3
            && self
                .currency_code
                .chars()
                .all(|character| character.is_ascii_uppercase());
        if !currency_is_valid {
            errors.insert(
                "currencyCode".to_owned(),
                "Use a three-letter uppercase currency code.".to_owned(),
            );
        }
        if !(0..=3).contains(&self.currency_exponent) {
            errors.insert(
                "currencyExponent".to_owned(),
                "Currency exponent must be between 0 and 3.".to_owned(),
            );
        }
        if self.application_timezone.parse::<Tz>().is_err() {
            errors.insert(
                "applicationTimezone".to_owned(),
                "Use a valid IANA timezone.".to_owned(),
            );
        }
        if !matches!(
            self.date_format.as_str(),
            "DD/MM/YYYY" | "MM/DD/YYYY" | "YYYY-MM-DD"
        ) {
            errors.insert(
                "dateFormat".to_owned(),
                "Choose a supported date format.".to_owned(),
            );
        }
        if !matches!(self.time_format.as_str(), "12h" | "24h") {
            errors.insert(
                "timeFormat".to_owned(),
                "Choose a supported time format.".to_owned(),
            );
        }
        if !matches!(self.first_weekday, 1 | 7) {
            errors.insert(
                "firstWeekday".to_owned(),
                "Choose Monday or Sunday.".to_owned(),
            );
        }
        if !matches!(self.theme.as_str(), "system" | "light" | "dark") {
            errors.insert("theme".to_owned(), "Choose a supported theme.".to_owned());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::validation(errors))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateSettingsInput;

    fn valid_input() -> UpdateSettingsInput {
        UpdateSettingsInput {
            currency_code: "VND".to_owned(),
            currency_exponent: 0,
            application_timezone: "Asia/Ho_Chi_Minh".to_owned(),
            date_format: "DD/MM/YYYY".to_owned(),
            time_format: "24h".to_owned(),
            first_weekday: 1,
            theme: "system".to_owned(),
        }
    }

    #[test]
    fn accepts_supported_settings() {
        assert!(valid_input().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_fields_together() {
        let mut input = valid_input();
        input.currency_code = "vnd".to_owned();
        input.application_timezone = "Local time".to_owned();
        input.first_weekday = 3;

        let error = input.validate().expect_err("invalid settings");
        let fields = error.field_errors.expect("field errors");
        assert!(fields.contains_key("currencyCode"));
        assert!(fields.contains_key("applicationTimezone"));
        assert!(fields.contains_key("firstWeekday"));
    }
}
