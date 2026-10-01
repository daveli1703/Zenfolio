use crate::error::AppError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATUSES: [&str; 4] = ["active", "completed", "paused", "abandoned"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_value: String,
    pub current_value: String,
    pub decimal_scale: u8,
    pub unit: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
    pub completed_at: Option<String>,
    pub progress_basis_points: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub title: String,
    pub description: Option<String>,
    pub target_value: String,
    pub current_value: String,
    pub decimal_scale: u8,
    pub unit: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalProgressInput {
    pub id: String,
    pub current_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalStatusInput {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoalFilters {
    pub status: Option<String>,
    pub sort_field: Option<String>,
    pub sort_direction: Option<String>,
}

impl GoalInput {
    pub fn normalize(mut self) -> Result<(Self, i64, i64), AppError> {
        self.title = self.title.trim().into();
        self.description = optional(self.description);
        self.unit = self.unit.trim().into();
        self.end_date = optional(self.end_date);
        let target = parse_i64(&self.target_value, "targetValue")?;
        let current = parse_i64(&self.current_value, "currentValue")?;
        let mut e = BTreeMap::new();
        if self.title.is_empty() {
            e.insert("title".into(), "Goal title is required.".into());
        }
        if target <= 0 {
            e.insert(
                "targetValue".into(),
                "Target must be greater than zero.".into(),
            );
        }
        if current < 0 {
            e.insert("currentValue".into(), "Progress cannot be negative.".into());
        }
        if self.decimal_scale > 3 {
            e.insert(
                "decimalScale".into(),
                "Use at most three decimal places.".into(),
            );
        }
        if self.unit.is_empty() {
            e.insert("unit".into(), "Unit is required.".into());
        }
        if !STATUSES.contains(&self.status.as_str()) {
            e.insert("status".into(), "Choose a valid status.".into());
        }
        let start = parse_date(&self.start_date);
        if start.is_err() {
            e.insert("startDate".into(), "Use a valid start date.".into());
        }
        if let Some(end) = &self.end_date {
            match (start, parse_date(end)) {
                (_, Err(_)) => {
                    e.insert("endDate".into(), "Use a valid end date.".into());
                }
                (Ok(s), Ok(v)) if v < s => {
                    e.insert(
                        "endDate".into(),
                        "End date cannot be before start date.".into(),
                    );
                }
                _ => {}
            }
        }
        valid(e)?;
        self.target_value = target.to_string();
        self.current_value = current.to_string();
        Ok((self, target, current))
    }
}
impl GoalFilters {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.status = optional(self.status);
        self.sort_field = optional(self.sort_field).or(Some("endDate".into()));
        self.sort_direction = optional(self.sort_direction).or(Some("asc".into()));
        let mut e = BTreeMap::new();
        if self
            .status
            .as_ref()
            .is_some_and(|s| !STATUSES.contains(&s.as_str()))
        {
            e.insert("status".into(), "Choose a valid status filter.".into());
        }
        if !matches!(
            self.sort_field.as_deref(),
            Some("endDate" | "createdAt" | "progress" | "title")
        ) {
            e.insert("sortField".into(), "Choose a valid sort field.".into());
        }
        if !matches!(self.sort_direction.as_deref(), Some("asc" | "desc")) {
            e.insert(
                "sortDirection".into(),
                "Choose a valid sort direction.".into(),
            );
        }
        valid(e)?;
        Ok(self)
    }
    pub fn order_by(&self) -> &'static str {
        match (self.sort_field.as_deref(), self.sort_direction.as_deref()) {
            (Some("createdAt"), Some("asc")) => "created_at ASC,id ASC",
            (Some("createdAt"), Some("desc")) => "created_at DESC,id DESC",
            (Some("progress"), Some("asc")) => {
                "(CAST(current_value AS REAL)/target_value) ASC,title COLLATE NOCASE"
            }
            (Some("progress"), Some("desc")) => {
                "(CAST(current_value AS REAL)/target_value) DESC,title COLLATE NOCASE"
            }
            (Some("title"), Some("desc")) => "title COLLATE NOCASE DESC,id DESC",
            (Some("title"), _) => "title COLLATE NOCASE ASC,id ASC",
            (Some("endDate"), Some("desc")) => "end_date IS NULL ASC,end_date DESC,created_at DESC",
            _ => "end_date IS NULL ASC,end_date ASC,created_at DESC",
        }
    }
}
pub fn progress_basis_points(current: i64, target: i64) -> Result<String, AppError> {
    let value = (i128::from(current))
        .checked_mul(10_000)
        .ok_or_else(AppError::internal)?
        / i128::from(target);
    Ok(value.to_string())
}
fn parse_i64(value: &str, field: &str) -> Result<i64, AppError> {
    value.parse::<i64>().map_err(|_| {
        let mut e = BTreeMap::new();
        e.insert(field.into(), "Value is outside the supported range.".into());
        AppError::validation(e)
    })
}
fn parse_date(v: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(v, "%Y-%m-%d")
}
fn optional(v: Option<String>) -> Option<String> {
    v.and_then(|v| {
        let v = v.trim().to_owned();
        (!v.is_empty()).then_some(v)
    })
}
fn valid(e: BTreeMap<String, String>) -> Result<(), AppError> {
    if e.is_empty() {
        Ok(())
    } else {
        Err(AppError::validation(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> GoalInput {
        GoalInput {
            title: "Read".into(),
            description: None,
            target_value: "200".into(),
            current_value: "125".into(),
            decimal_scale: 1,
            unit: "books".into(),
            start_date: "2026-01-01".into(),
            end_date: Some("2026-12-31".into()),
            status: "active".into(),
        }
    }
    #[test]
    fn validates_values_dates_and_scale() {
        let mut v = input();
        v.title = " ".into();
        v.target_value = "0".into();
        v.current_value = "-1".into();
        v.decimal_scale = 4;
        v.end_date = Some("2025-01-01".into());
        let f = v.normalize().unwrap_err().field_errors.unwrap();
        for k in [
            "title",
            "targetValue",
            "currentValue",
            "decimalScale",
            "endDate",
        ] {
            assert!(f.contains_key(k));
        }
    }
    #[test]
    fn exact_percentage_supports_over_target() {
        assert_eq!(progress_basis_points(75, 100).unwrap(), "7500");
        assert_eq!(progress_basis_points(120, 100).unwrap(), "12000");
    }
    #[test]
    fn allows_all_supported_scales() {
        for scale in 0..=3 {
            let mut v = input();
            v.decimal_scale = scale;
            assert!(v.normalize().is_ok());
        }
    }
    #[test]
    fn sort_fields_are_allowlisted() {
        assert!(GoalFilters {
            sort_field: Some("DROP TABLE".into()),
            ..Default::default()
        }
        .normalize()
        .is_err());
    }
}
