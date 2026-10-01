use crate::{
    db::{goals_repository, goals_repository::GoalWrite, migrations::now_utc, Database},
    domain::goals::{Goal, GoalFilters, GoalInput, GoalProgressInput, GoalStatusInput, STATUSES},
    error::AppError,
};
use std::collections::BTreeMap;
use uuid::Uuid;

pub async fn list(database: &Database, filters: GoalFilters) -> Result<Vec<Goal>, AppError> {
    let filters = filters.normalize()?;
    database
        .with_connection(move |c| goals_repository::list(c, &filters))
        .await
}
pub async fn get(database: &Database, id: String) -> Result<Goal, AppError> {
    database
        .with_connection(move |c| goals_repository::get(c, &id)?.ok_or_else(not_found))
        .await
}
pub async fn create(database: &Database, input: GoalInput) -> Result<Goal, AppError> {
    let (input, target, current) = input.normalize()?;
    database
        .with_connection(move |c| {
            let now = now_utc();
            let completed = (input.status == "completed").then(|| now.clone());
            let id = Uuid::new_v4().to_string();
            goals_repository::insert(
                c,
                &write(
                    &id,
                    &input,
                    target,
                    current,
                    completed.as_deref(),
                    &now,
                    &now,
                ),
            )
        })
        .await
}
pub async fn update(database: &Database, id: String, input: GoalInput) -> Result<Goal, AppError> {
    let (input, target, current) = input.normalize()?;
    database
        .with_connection(move |c| {
            let old = goals_repository::get(c, &id)?.ok_or_else(not_found)?;
            let now = now_utc();
            let completed = completion_timestamp(&old, &input.status, &now);
            goals_repository::update(
                c,
                &write(
                    &id,
                    &input,
                    target,
                    current,
                    completed.as_deref(),
                    &old.created_at,
                    &now,
                ),
            )?
            .ok_or_else(not_found)
        })
        .await
}
pub async fn update_progress(
    database: &Database,
    input: GoalProgressInput,
) -> Result<Goal, AppError> {
    let value = parse_progress(&input.current_value)?;
    database
        .with_connection(move |c| {
            if goals_repository::get(c, &input.id)?.is_none() {
                return Err(not_found());
            }
            let now = now_utc();
            goals_repository::update_progress(c, &input.id, value, &now)?;
            goals_repository::get(c, &input.id)?.ok_or_else(not_found)
        })
        .await
}
pub async fn update_status(database: &Database, input: GoalStatusInput) -> Result<Goal, AppError> {
    if !STATUSES.contains(&input.status.as_str()) {
        return Err(field("status", "Choose a valid status."));
    }
    database
        .with_connection(move |c| {
            let old = goals_repository::get(c, &input.id)?.ok_or_else(not_found)?;
            let now = now_utc();
            let completed = completion_timestamp(&old, &input.status, &now);
            goals_repository::update_status(
                c,
                &input.id,
                &input.status,
                completed.as_deref(),
                &now,
            )?;
            goals_repository::get(c, &input.id)?.ok_or_else(not_found)
        })
        .await
}
pub async fn delete(database: &Database, id: String) -> Result<(), AppError> {
    database
        .with_connection(move |c| {
            if goals_repository::delete(c, &id)? {
                Ok(())
            } else {
                Err(not_found())
            }
        })
        .await
}

fn write<'a>(
    id: &'a str,
    input: &'a GoalInput,
    target: i64,
    current: i64,
    completed_at: Option<&'a str>,
    created_at: &'a str,
    updated_at: &'a str,
) -> GoalWrite<'a> {
    GoalWrite {
        id,
        title: &input.title,
        description: input.description.as_deref(),
        target_value: target,
        current_value: current,
        decimal_scale: input.decimal_scale,
        unit: &input.unit,
        start_date: &input.start_date,
        end_date: input.end_date.as_deref(),
        status: &input.status,
        completed_at,
        created_at,
        updated_at,
    }
}
fn completion_timestamp(goal: &Goal, status: &str, now: &str) -> Option<String> {
    if status == "completed" {
        goal.completed_at.clone().or_else(|| Some(now.to_owned()))
    } else {
        None
    }
}
fn parse_progress(value: &str) -> Result<i64, AppError> {
    match value.parse::<i64>() {
        Ok(value) if value >= 0 => Ok(value),
        _ => Err(field(
            "currentValue",
            "Progress must be a nonnegative supported value.",
        )),
    }
}
fn field(name: &str, message: &str) -> AppError {
    let mut fields = BTreeMap::new();
    fields.insert(name.to_owned(), message.to_owned());
    AppError::validation(fields)
}
fn not_found() -> AppError {
    AppError::new("GOAL_NOT_FOUND", "The goal no longer exists.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DataEnvironment, StoragePaths};
    use tempfile::tempdir;
    fn input() -> GoalInput {
        GoalInput {
            title: "Run".into(),
            description: None,
            target_value: "10000".into(),
            current_value: "2500".into(),
            decimal_scale: 2,
            unit: "km".into(),
            start_date: "2026-01-01".into(),
            end_date: Some("2026-12-31".into()),
            status: "active".into(),
        }
    }
    #[test]
    fn goal_flow_is_exact_and_persistent() {
        tauri::async_runtime::block_on(async {
            let dir = tempdir().unwrap();
            let paths = StoragePaths::new(dir.path().to_owned(), DataEnvironment::Test);
            let db = Database::initialize(paths.clone()).unwrap();
            let goal = create(&db, input()).await.unwrap();
            assert_eq!(goal.progress_basis_points, "2500");
            let progressed = update_progress(
                &db,
                GoalProgressInput {
                    id: goal.id.clone(),
                    current_value: "12500".into(),
                },
            )
            .await
            .unwrap();
            assert_eq!(progressed.progress_basis_points, "12500");
            let done = update_status(
                &db,
                GoalStatusInput {
                    id: goal.id.clone(),
                    status: "completed".into(),
                },
            )
            .await
            .unwrap();
            assert!(done.completed_at.is_some());
            let reopened = update_status(
                &db,
                GoalStatusInput {
                    id: goal.id.clone(),
                    status: "active".into(),
                },
            )
            .await
            .unwrap();
            assert!(reopened.completed_at.is_none());
            drop(db);
            let db = Database::initialize(paths).unwrap();
            assert_eq!(get(&db, goal.id).await.unwrap().current_value, "12500");
        });
    }
}
