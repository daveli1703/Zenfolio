use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const DAILY_MASK: i64 = 127;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub unit: Option<String>,
    pub color: String,
    pub start_date: String,
    pub archive_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HabitRule {
    pub id: String,
    pub habit_id: String,
    pub effective_date: String,
    pub target: i64,
    pub weekday_mask: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HabitEntry {
    pub id: String,
    pub habit_id: String,
    pub date: String,
    pub value: i64,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitDetail {
    pub habit: Habit,
    pub rules: Vec<HabitRule>,
    pub has_entries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHabitInput {
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub unit: Option<String>,
    pub color: String,
    pub start_date: String,
    pub target: i64,
    pub weekday_mask: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitProfileInput {
    pub name: String,
    pub description: Option<String>,
    pub target_type: String,
    pub unit: Option<String>,
    pub color: String,
    pub start_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitRuleInput {
    pub target: i64,
    pub weekday_mask: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitEntryInput {
    pub habit_id: String,
    pub date: String,
    pub value: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitDay {
    pub date: String,
    pub state: String,
    pub value: i64,
    pub notes: Option<String>,
    pub target: Option<i64>,
    pub completed: bool,
    pub intensity: u8,
    pub progress_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitToday {
    pub habit: Habit,
    pub day: HabitDay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HabitStatistics {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub completed_days: u32,
    pub elapsed_scheduled_days: u32,
    pub completion_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitYear {
    pub habit: Habit,
    pub year: i32,
    pub days: Vec<HabitDay>,
    pub statistics: HabitStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitDeleteImpact {
    pub rule_count: i64,
    pub entry_count: i64,
}

impl CreateHabitInput {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.name = self.name.trim().to_owned();
        self.description = normalize_optional(self.description);
        self.unit = normalize_optional(self.unit);
        self.color = self.color.trim().to_owned();
        validate_common(
            &self.name,
            &self.target_type,
            &self.unit,
            &self.color,
            &self.start_date,
        )?;
        validate_rule(&self.target_type, self.target, self.weekday_mask)?;
        Ok(self)
    }
}

impl HabitProfileInput {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.name = self.name.trim().to_owned();
        self.description = normalize_optional(self.description);
        self.unit = normalize_optional(self.unit);
        self.color = self.color.trim().to_owned();
        validate_common(
            &self.name,
            &self.target_type,
            &self.unit,
            &self.color,
            &self.start_date,
        )?;
        Ok(self)
    }
}

impl HabitRuleInput {
    pub fn validate(&self, target_type: &str) -> Result<(), AppError> {
        validate_rule(target_type, self.target, self.weekday_mask)
    }
}

fn validate_common(
    name: &str,
    target_type: &str,
    unit: &Option<String>,
    color: &str,
    start_date: &str,
) -> Result<(), AppError> {
    let mut errors = BTreeMap::new();
    if name.is_empty() {
        errors.insert("name".into(), "Habit name is required.".into());
    }
    if !matches!(target_type, "boolean" | "count" | "duration") {
        errors.insert("targetType".into(), "Choose a valid target type.".into());
    }
    if target_type == "count" && unit.is_none() {
        errors.insert("unit".into(), "Count habits require a unit.".into());
    }
    if target_type != "count" && unit.is_some() {
        errors.insert("unit".into(), "Only count habits use a custom unit.".into());
    }
    if !valid_color(color) {
        errors.insert("color".into(), "Use a six-digit hex color.".into());
    }
    if parse_date(start_date).is_err() {
        errors.insert("startDate".into(), "Use a valid start date.".into());
    }
    validation_result(errors)
}

fn validate_rule(target_type: &str, target: i64, weekday_mask: i64) -> Result<(), AppError> {
    let mut errors = BTreeMap::new();
    if target <= 0 || (target_type == "boolean" && target != 1) {
        errors.insert(
            "target".into(),
            if target_type == "boolean" {
                "Boolean habits must target one completion."
            } else {
                "Target must be a positive whole number."
            }
            .into(),
        );
    }
    if !(1..=DAILY_MASK).contains(&weekday_mask) {
        errors.insert("weekdayMask".into(), "Choose at least one weekday.".into());
    }
    validation_result(errors)
}

pub fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::new("VALIDATION_ERROR", "Use a valid calendar date."))
}

pub fn is_scheduled(date: NaiveDate, mask: i64) -> bool {
    let bit = date.weekday().num_days_from_monday();
    mask & (1_i64 << bit) != 0
}

pub fn applicable_rule(rules: &[HabitRule], date: NaiveDate) -> Option<&HabitRule> {
    rules
        .iter()
        .rev()
        .find(|rule| parse_date(&rule.effective_date).is_ok_and(|effective| effective <= date))
}

pub fn project_day(
    habit: &Habit,
    rules: &[HabitRule],
    entries: &[HabitEntry],
    date: NaiveDate,
    today: NaiveDate,
) -> HabitDay {
    let start = parse_date(&habit.start_date).expect("stored start date must be valid");
    let archive = habit
        .archive_date
        .as_deref()
        .and_then(|value| parse_date(value).ok());
    let entry = entries
        .iter()
        .find(|entry| entry.date == date.format("%Y-%m-%d").to_string());
    let value = entry.map_or(0, |entry| entry.value);
    let rule = applicable_rule(rules, date);
    let state = if date < start {
        "pre_start"
    } else if archive.is_some_and(|boundary| date >= boundary) {
        "post_archive"
    } else if date > today {
        "future"
    } else if rule.is_none_or(|rule| !is_scheduled(date, rule.weekday_mask)) {
        "unscheduled"
    } else {
        "eligible"
    };
    let target = rule.map(|rule| rule.target);
    let completed = state == "eligible" && target.is_some_and(|target| value >= target);
    let intensity = if state != "eligible" {
        0
    } else {
        intensity(value, target.unwrap_or(1))
    };
    HabitDay {
        date: date.format("%Y-%m-%d").to_string(),
        state: state.into(),
        value,
        notes: entry.and_then(|entry| entry.notes.clone()),
        target,
        completed,
        intensity,
        progress_ratio: target.map(|target| value as f64 / target as f64),
    }
}

pub fn intensity(value: i64, target: i64) -> u8 {
    if value <= 0 {
        0
    } else if value.saturating_mul(4) < target {
        1
    } else if value.saturating_mul(2) < target {
        2
    } else if value < target {
        3
    } else {
        4
    }
}

pub fn statistics(days: &[HabitDay], today: NaiveDate) -> HabitStatistics {
    let elapsed: Vec<&HabitDay> = days
        .iter()
        .filter(|day| {
            day.state == "eligible"
                && parse_date(&day.date)
                    .is_ok_and(|date| date < today || (date == today && day.completed))
        })
        .collect();
    let completed_days = elapsed.iter().filter(|day| day.completed).count() as u32;
    let mut run = 0_u32;
    let mut longest = 0_u32;
    for day in &elapsed {
        if day.completed {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let current_streak = elapsed.iter().rev().take_while(|day| day.completed).count() as u32;
    let denominator = elapsed.len() as u32;
    HabitStatistics {
        current_streak,
        longest_streak: longest,
        completed_days,
        elapsed_scheduled_days: denominator,
        completion_rate: (denominator > 0).then_some(completed_days as f64 / denominator as f64),
    }
}

pub fn year_dates(year: i32) -> Result<Vec<NaiveDate>, AppError> {
    if !(1..=9999).contains(&year) {
        return Err(AppError::new("VALIDATION_ERROR", "Choose a valid year."));
    }
    let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap_or(NaiveDate::MAX);
    let mut dates = Vec::new();
    let mut date = start;
    while date < end {
        dates.push(date);
        date += Duration::days(1);
    }
    Ok(dates)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}
fn valid_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}
fn validation_result(errors: BTreeMap<String, String>) -> Result<(), AppError> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::validation(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn habit() -> Habit {
        Habit {
            id: "h".into(),
            name: "Read".into(),
            description: None,
            target_type: "count".into(),
            unit: Some("pages".into()),
            color: "#17735a".into(),
            start_date: "2024-01-01".into(),
            archive_date: None,
            created_at: "now".into(),
            updated_at: "now".into(),
        }
    }
    fn rule(date: &str, target: i64, mask: i64) -> HabitRule {
        HabitRule {
            id: date.into(),
            habit_id: "h".into(),
            effective_date: date.into(),
            target,
            weekday_mask: mask,
            created_at: "now".into(),
            updated_at: "now".into(),
        }
    }
    fn entry(date: &str, value: i64) -> HabitEntry {
        HabitEntry {
            id: date.into(),
            habit_id: "h".into(),
            date: date.into(),
            value,
            notes: None,
            created_at: "now".into(),
            updated_at: "now".into(),
        }
    }

    #[test]
    fn weekday_masks_use_monday_as_bit_zero() {
        assert!(is_scheduled(parse_date("2024-01-01").unwrap(), 1));
        assert!(!is_scheduled(parse_date("2024-01-02").unwrap(), 1));
    }
    #[test]
    fn historical_rule_and_intensity_are_date_specific() {
        let rules = vec![rule("2024-01-01", 10, 127), rule("2024-02-01", 20, 127)];
        let entries = vec![entry("2024-01-31", 10), entry("2024-02-01", 10)];
        assert_eq!(
            project_day(
                &habit(),
                &rules,
                &entries,
                parse_date("2024-01-31").unwrap(),
                parse_date("2024-02-02").unwrap()
            )
            .intensity,
            4
        );
        assert_eq!(
            project_day(
                &habit(),
                &rules,
                &entries,
                parse_date("2024-02-01").unwrap(),
                parse_date("2024-02-02").unwrap()
            )
            .intensity,
            3
        );
    }
    #[test]
    fn unfinished_today_does_not_break_a_streak() {
        let rules = vec![rule("2024-01-01", 1, 127)];
        let entries = vec![entry("2024-01-01", 1), entry("2024-01-02", 1)];
        let today = parse_date("2024-01-03").unwrap();
        let days = year_dates(2024)
            .unwrap()
            .into_iter()
            .take(3)
            .map(|date| project_day(&habit(), &rules, &entries, date, today))
            .collect::<Vec<_>>();
        let stats = statistics(&days, today);
        assert_eq!(stats.current_streak, 2);
        assert_eq!(stats.elapsed_scheduled_days, 2);
    }
    #[test]
    fn missed_past_opportunity_breaks_streak_but_unscheduled_gap_does_not() {
        let rules = vec![rule("2024-01-01", 1, 0b0010101)];
        let entries = vec![entry("2024-01-01", 1), entry("2024-01-05", 1)];
        let today = parse_date("2024-01-06").unwrap();
        let days = year_dates(2024)
            .unwrap()
            .into_iter()
            .take(6)
            .map(|date| project_day(&habit(), &rules, &entries, date, today))
            .collect::<Vec<_>>();
        let stats = statistics(&days, today);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
    }
    #[test]
    fn leap_year_has_366_real_dates() {
        assert_eq!(year_dates(2024).unwrap().len(), 366);
        assert_eq!(year_dates(2025).unwrap().len(), 365);
    }
    #[test]
    fn intensity_thresholds_are_exact() {
        assert_eq!(
            (
                intensity(0, 100),
                intensity(24, 100),
                intensity(25, 100),
                intensity(50, 100),
                intensity(100, 100)
            ),
            (0, 1, 2, 3, 4)
        );
    }
    #[test]
    fn archive_boundary_and_empty_completion_rate_are_explicit() {
        let mut value = habit();
        value.archive_date = Some("2024-01-03".into());
        let rules = vec![rule("2024-01-01", 1, 1)];
        let today = parse_date("2024-01-04").unwrap();
        assert_eq!(
            project_day(
                &value,
                &rules,
                &[],
                parse_date("2024-01-03").unwrap(),
                today
            )
            .state,
            "post_archive"
        );
        let only_today = vec![project_day(
            &habit(),
            &rules,
            &[],
            parse_date("2024-01-01").unwrap(),
            parse_date("2024-01-01").unwrap(),
        )];
        assert_eq!(
            statistics(&only_today, parse_date("2024-01-01").unwrap()).completion_rate,
            None
        );
    }
    #[test]
    fn validates_boolean_count_and_duration_inputs() {
        let boolean = CreateHabitInput {
            name: "Done".into(),
            description: None,
            target_type: "boolean".into(),
            unit: None,
            color: "#17735a".into(),
            start_date: "2026-09-30".into(),
            target: 1,
            weekday_mask: 127,
        };
        assert!(boolean.normalize().is_ok());
        let count = CreateHabitInput {
            name: "Read".into(),
            description: None,
            target_type: "count".into(),
            unit: Some("pages".into()),
            color: "#17735a".into(),
            start_date: "2026-09-30".into(),
            target: 10,
            weekday_mask: 31,
        };
        assert!(count.normalize().is_ok());
        let duration = CreateHabitInput {
            name: "Walk".into(),
            description: None,
            target_type: "duration".into(),
            unit: None,
            color: "#17735a".into(),
            start_date: "2026-09-30".into(),
            target: 30,
            weekday_mask: 127,
        };
        assert!(duration.normalize().is_ok());
    }
}
