use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use rusqlite::TransactionBehavior;
use uuid::Uuid;

use crate::{
    db::{
        habits_repository::{self, HabitWrite, RuleWrite},
        migrations::now_utc,
        settings_repository, Database,
    },
    domain::habits::{
        applicable_rule, is_scheduled, parse_date, project_day, statistics, year_dates,
        CreateHabitInput, Habit, HabitDeleteImpact, HabitDetail, HabitEntry, HabitEntryInput,
        HabitProfileInput, HabitRuleInput, HabitToday, HabitYear,
    },
    error::AppError,
};

pub async fn list(database: &Database) -> Result<Vec<Habit>, AppError> {
    database
        .with_connection(|connection| habits_repository::list(connection))
        .await
}

pub async fn detail(database: &Database, id: String) -> Result<HabitDetail, AppError> {
    database
        .with_connection(move |connection| detail_sync(connection, &id))
        .await
}

pub async fn create(database: &Database, input: CreateHabitInput) -> Result<HabitDetail, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let timestamp = now_utc();
            let id = Uuid::new_v4().to_string();
            let rule_id = Uuid::new_v4().to_string();
            habits_repository::insert_with_rule(
                connection,
                &HabitWrite {
                    id: &id,
                    name: &input.name,
                    description: input.description.as_deref(),
                    target_type: &input.target_type,
                    unit: input.unit.as_deref(),
                    color: &input.color,
                    start_date: &input.start_date,
                    archive_date: None,
                    created_at: &timestamp,
                    updated_at: &timestamp,
                },
                &RuleWrite {
                    id: &rule_id,
                    habit_id: &id,
                    effective_date: &input.start_date,
                    target: input.target,
                    weekday_mask: input.weekday_mask,
                    created_at: &timestamp,
                    updated_at: &timestamp,
                },
            )?;
            detail_sync(connection, &id)
        })
        .await
}

pub async fn update_profile(
    database: &Database,
    id: String,
    input: HabitProfileInput,
) -> Result<HabitDetail, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let existing = habits_repository::get(connection, &id)?.ok_or_else(not_found)?;
            let has_entries = habits_repository::has_entries(connection, &id)?;
            if has_entries
                && (input.target_type != existing.target_type
                    || input.unit != existing.unit
                    || input.start_date != existing.start_date)
            {
                return field_error(
                    "targetType",
                    "Type, unit, and start date are locked after logging begins.",
                );
            }
            let timestamp = now_utc();
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            habits_repository::update_profile(
                &transaction,
                &HabitWrite {
                    id: &id,
                    name: &input.name,
                    description: input.description.as_deref(),
                    target_type: &input.target_type,
                    unit: input.unit.as_deref(),
                    color: &input.color,
                    start_date: &input.start_date,
                    archive_date: existing.archive_date.as_deref(),
                    created_at: &existing.created_at,
                    updated_at: &timestamp,
                },
            )?;
            if !has_entries && input.start_date != existing.start_date {
                habits_repository::update_initial_rule_date(
                    &transaction,
                    &id,
                    &existing.start_date,
                    &input.start_date,
                    &timestamp,
                )?;
            }
            if !has_entries && input.target_type == "boolean" {
                habits_repository::normalize_boolean_rules(&transaction, &id, &timestamp)?;
            }
            transaction.commit()?;
            detail_sync(connection, &id)
        })
        .await
}

pub async fn schedule_rule(
    database: &Database,
    id: String,
    input: HabitRuleInput,
) -> Result<HabitDetail, AppError> {
    database
        .with_connection(move |connection| {
            let current_date = today(connection)?;
            schedule_rule_at(connection, &id, input, current_date)
        })
        .await
}

fn schedule_rule_at(
    connection: &rusqlite::Connection,
    id: &str,
    input: HabitRuleInput,
    current_date: NaiveDate,
) -> Result<HabitDetail, AppError> {
    let habit = habits_repository::get(connection, id)?.ok_or_else(not_found)?;
    input.validate(&habit.target_type)?;
    let tomorrow = current_date + Duration::days(1);
    if habit
        .archive_date
        .as_deref()
        .and_then(|d| parse_date(d).ok())
        .is_some_and(|archive| tomorrow >= archive)
    {
        return field_error("schedule", "Archived habits cannot receive new rules.");
    }
    let effective = tomorrow.format("%Y-%m-%d").to_string();
    let timestamp = now_utc();
    let rule_id = Uuid::new_v4().to_string();
    habits_repository::upsert_rule(
        connection,
        &RuleWrite {
            id: &rule_id,
            habit_id: id,
            effective_date: &effective,
            target: input.target,
            weekday_mask: input.weekday_mask,
            created_at: &timestamp,
            updated_at: &timestamp,
        },
    )?;
    detail_sync(connection, id)
}

pub async fn archive(database: &Database, id: String) -> Result<HabitDetail, AppError> {
    database
        .with_connection(move |connection| {
            let existing = habits_repository::get(connection, &id)?.ok_or_else(not_found)?;
            if existing.archive_date.is_none() {
                let date = (today(connection)? + Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string();
                habits_repository::archive(connection, &id, &date, &now_utc())?;
            }
            detail_sync(connection, &id)
        })
        .await
}

pub async fn save_entry(
    database: &Database,
    mut input: HabitEntryInput,
) -> Result<HabitEntry, AppError> {
    input.habit_id = input.habit_id.trim().to_owned();
    input.notes = input.notes.and_then(|v| {
        let v = v.trim().to_owned();
        (!v.is_empty()).then_some(v)
    });
    database
        .with_connection(move |connection| save_entry_at(connection, input, today(connection)?))
        .await
}

fn save_entry_at(
    connection: &rusqlite::Connection,
    input: HabitEntryInput,
    today: NaiveDate,
) -> Result<HabitEntry, AppError> {
    let habit = habits_repository::get(connection, &input.habit_id)?.ok_or_else(not_found)?;
    let date =
        parse_date(&input.date).map_err(|_| validation("date", "Use a valid entry date."))?;
    if input.value < 0 {
        return field_error("value", "Value cannot be negative.");
    }
    if date > today {
        return field_error("date", "Future entries are not allowed.");
    }
    if date < parse_date(&habit.start_date)? {
        return field_error("date", "This date is before the habit started.");
    }
    if habit
        .archive_date
        .as_deref()
        .and_then(|d| parse_date(d).ok())
        .is_some_and(|archive| date >= archive)
    {
        return field_error("date", "This date is after the habit was archived.");
    }
    let rules = habits_repository::rules(connection, &habit.id)?;
    let rule = applicable_rule(&rules, date)
        .ok_or_else(|| validation("date", "No habit rule applies to this date."))?;
    if !is_scheduled(date, rule.weekday_mask) {
        return field_error("date", "This habit is not scheduled for that date.");
    }
    if habit.target_type == "boolean" && !matches!(input.value, 0 | 1) {
        return field_error("value", "Boolean habit values must be zero or one.");
    }
    let timestamp = now_utc();
    let existing =
        habits_repository::entries(connection, &habit.id, Some(&input.date), Some(&input.date))?
            .into_iter()
            .next();
    let entry = HabitEntry {
        id: existing
            .as_ref()
            .map_or_else(|| Uuid::new_v4().to_string(), |v| v.id.clone()),
        habit_id: habit.id,
        date: input.date,
        value: input.value,
        notes: input.notes,
        created_at: existing
            .as_ref()
            .map_or_else(|| timestamp.clone(), |v| v.created_at.clone()),
        updated_at: timestamp,
    };
    habits_repository::upsert_entry(connection, &entry)?;
    Ok(entry)
}

pub async fn today_list(database: &Database) -> Result<Vec<HabitToday>, AppError> {
    database
        .with_connection(|connection| {
            let today = today(connection)?;
            let mut result = Vec::new();
            for habit in habits_repository::list(connection)? {
                if parse_date(&habit.start_date)? > today
                    || habit
                        .archive_date
                        .as_deref()
                        .and_then(|d| parse_date(d).ok())
                        .is_some_and(|d| d <= today)
                {
                    continue;
                }
                let rules = habits_repository::rules(connection, &habit.id)?;
                let date = today.format("%Y-%m-%d").to_string();
                let entries =
                    habits_repository::entries(connection, &habit.id, Some(&date), Some(&date))?;
                result.push(HabitToday {
                    day: project_day(&habit, &rules, &entries, today, today),
                    habit,
                });
            }
            Ok(result)
        })
        .await
}

pub async fn year(database: &Database, id: String, year: i32) -> Result<HabitYear, AppError> {
    database
        .with_connection(move |connection| {
            let habit = habits_repository::get(connection, &id)?.ok_or_else(not_found)?;
            let today = today(connection)?;
            let rules = habits_repository::rules(connection, &id)?;
            let from = format!("{year:04}-01-01");
            let to = format!("{year:04}-12-31");
            let entries = habits_repository::entries(connection, &id, Some(&from), Some(&to))?;
            let days = year_dates(year)?
                .into_iter()
                .map(|date| project_day(&habit, &rules, &entries, date, today))
                .collect::<Vec<_>>();
            let start = parse_date(&habit.start_date)?;
            let lifetime_end = today.max(start);
            let all_entries = habits_repository::entries(
                connection,
                &id,
                None,
                Some(&today.format("%Y-%m-%d").to_string()),
            )?;
            let mut date = start;
            let mut lifetime = Vec::new();
            while date <= lifetime_end {
                lifetime.push(project_day(&habit, &rules, &all_entries, date, today));
                date += Duration::days(1);
            }
            Ok(HabitYear {
                habit,
                year,
                days,
                statistics: statistics(&lifetime, today),
            })
        })
        .await
}

pub async fn delete_impact(database: &Database, id: String) -> Result<HabitDeleteImpact, AppError> {
    database
        .with_connection(move |connection| {
            habits_repository::delete_impact(connection, &id)?.ok_or_else(not_found)
        })
        .await
}
pub async fn delete(database: &Database, id: String) -> Result<(), AppError> {
    database
        .with_connection(move |connection| {
            if habits_repository::delete(connection, &id)? {
                Ok(())
            } else {
                Err(not_found())
            }
        })
        .await
}

fn detail_sync(connection: &rusqlite::Connection, id: &str) -> Result<HabitDetail, AppError> {
    let habit = habits_repository::get(connection, id)?.ok_or_else(not_found)?;
    Ok(HabitDetail {
        rules: habits_repository::rules(connection, id)?,
        has_entries: habits_repository::has_entries(connection, id)?,
        habit,
    })
}
fn today(connection: &rusqlite::Connection) -> Result<NaiveDate, AppError> {
    let value = settings_repository::get(connection)?.application_timezone;
    let timezone: Tz = value.parse().map_err(|_| AppError::internal())?;
    Ok(Utc::now().with_timezone(&timezone).date_naive())
}
fn validation(field: &str, message: &str) -> AppError {
    let mut fields = BTreeMap::new();
    fields.insert(field.to_owned(), message.to_owned());
    AppError::validation(fields)
}
fn field_error<T>(field: &str, message: &str) -> Result<T, AppError> {
    Err(validation(field, message))
}
fn not_found() -> AppError {
    AppError::new("HABIT_NOT_FOUND", "The habit no longer exists.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DataEnvironment, StoragePaths};
    use tempfile::tempdir;
    fn input() -> CreateHabitInput {
        CreateHabitInput {
            name: "Read".into(),
            description: None,
            target_type: "count".into(),
            unit: Some("pages".into()),
            color: "#17735a".into(),
            start_date: "2026-09-28".into(),
            target: 10,
            weekday_mask: 127,
        }
    }
    #[test]
    fn workflow_persists_rules_entries_and_cascades_delete() {
        tauri::async_runtime::block_on(async {
            let dir = tempdir().unwrap();
            let db = Database::initialize(StoragePaths::new(
                dir.path().to_owned(),
                DataEnvironment::Test,
            ))
            .unwrap();
            let habit = create(&db, input()).await.unwrap();
            db.with_connection({
                let id = habit.habit.id.clone();
                move |c| {
                    save_entry_at(
                        c,
                        HabitEntryInput {
                            habit_id: id,
                            date: "2026-09-28".into(),
                            value: 10,
                            notes: None,
                        },
                        NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
                    )
                }
            })
            .await
            .unwrap();
            let impact = delete_impact(&db, habit.habit.id.clone()).await.unwrap();
            assert_eq!((impact.rule_count, impact.entry_count), (1, 1));
            delete(&db, habit.habit.id).await.unwrap();
        });
    }
    #[test]
    fn rejects_future_unscheduled_and_invalid_boolean_entries() {
        tauri::async_runtime::block_on(async {
            let dir = tempdir().unwrap();
            let db = Database::initialize(StoragePaths::new(
                dir.path().to_owned(),
                DataEnvironment::Test,
            ))
            .unwrap();
            let mut value = input();
            value.target_type = "boolean".into();
            value.unit = None;
            value.target = 1;
            value.weekday_mask = 1;
            let habit = create(&db, value).await.unwrap();
            let id = habit.habit.id;
            for (date, value) in [("2026-09-29", 1), ("2026-10-01", 1), ("2026-09-28", 2)] {
                let result = db
                    .with_connection({
                        let id = id.clone();
                        move |c| {
                            save_entry_at(
                                c,
                                HabitEntryInput {
                                    habit_id: id,
                                    date: date.into(),
                                    value,
                                    notes: None,
                                },
                                NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
                            )
                        }
                    })
                    .await;
                assert!(result.is_err());
            }
        });
    }

    #[test]
    fn tomorrow_rule_replaces_pending_change_and_preserves_history() {
        tauri::async_runtime::block_on(async {
            let dir = tempdir().unwrap();
            let db = Database::initialize(StoragePaths::new(
                dir.path().to_owned(),
                DataEnvironment::Test,
            ))
            .unwrap();
            let habit = create(&db, input()).await.unwrap();
            let id = habit.habit.id;
            let current = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
            let detail = db
                .with_connection(move |connection| {
                    schedule_rule_at(
                        connection,
                        &id,
                        HabitRuleInput {
                            target: 20,
                            weekday_mask: 31,
                        },
                        current,
                    )?;
                    schedule_rule_at(
                        connection,
                        &id,
                        HabitRuleInput {
                            target: 30,
                            weekday_mask: 21,
                        },
                        current,
                    )
                })
                .await
                .unwrap();
            assert_eq!(detail.rules.len(), 2);
            assert_eq!(detail.rules[0].target, 10);
            assert_eq!(detail.rules[1].effective_date, "2026-10-01");
            assert_eq!(
                (detail.rules[1].target, detail.rules[1].weekday_mask),
                (30, 21)
            );
        });
    }
}
