use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{
    db::{
        migrations::now_utc, projects_repository, tags_repository, tasks_repository,
        tasks_repository::TaskWrite, Database,
    },
    domain::tasks::{Task, TaskFilters, TaskInput, STATUSES},
    error::AppError,
};

pub async fn list(database: &Database, filters: TaskFilters) -> Result<Vec<Task>, AppError> {
    let filters = filters.normalize()?;
    database
        .with_connection(move |connection| tasks_repository::list(connection, &filters))
        .await
}

pub async fn get(database: &Database, id: String) -> Result<Task, AppError> {
    database
        .with_connection(move |connection| {
            tasks_repository::get(connection, &id)?.ok_or_else(not_found)
        })
        .await
}

pub async fn create(database: &Database, input: TaskInput) -> Result<Task, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            validate_project(connection, input.project_id.as_deref(), None)?;
            validate_tags(connection, &input.tag_ids)?;
            let timestamp = now_utc();
            let completed_at = (input.status == "done").then(|| timestamp.clone());
            let id = Uuid::new_v4().to_string();
            tasks_repository::insert(
                connection,
                &TaskWrite {
                    id: &id,
                    title: &input.title,
                    notes: input.notes.as_deref(),
                    due_date: input.due_date.as_deref(),
                    due_time: input.due_time.as_deref(),
                    priority: &input.priority,
                    status: &input.status,
                    project_id: input.project_id.as_deref(),
                    completed_at: completed_at.as_deref(),
                    created_at: &timestamp,
                    updated_at: &timestamp,
                    tag_ids: &input.tag_ids,
                },
            )
        })
        .await
}

pub async fn update(database: &Database, id: String, input: TaskInput) -> Result<Task, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let Some(existing) = tasks_repository::get(connection, &id)? else {
                return Err(not_found());
            };
            validate_project(
                connection,
                input.project_id.as_deref(),
                existing.project.as_ref().map(|project| project.id.as_str()),
            )?;
            validate_tags(connection, &input.tag_ids)?;
            let timestamp = now_utc();
            let completed_at = completion_timestamp(&existing, &input.status, &timestamp);
            tasks_repository::update(
                connection,
                &TaskWrite {
                    id: &id,
                    title: &input.title,
                    notes: input.notes.as_deref(),
                    due_date: input.due_date.as_deref(),
                    due_time: input.due_time.as_deref(),
                    priority: &input.priority,
                    status: &input.status,
                    project_id: input.project_id.as_deref(),
                    completed_at: completed_at.as_deref(),
                    created_at: &existing.created_at,
                    updated_at: &timestamp,
                    tag_ids: &input.tag_ids,
                },
            )?
            .ok_or_else(not_found)
        })
        .await
}

pub async fn set_status(database: &Database, id: String, status: String) -> Result<Task, AppError> {
    if !STATUSES.contains(&status.as_str()) {
        let mut fields = BTreeMap::new();
        fields.insert("status".to_owned(), "Choose a valid status.".to_owned());
        return Err(AppError::validation(fields));
    }
    database
        .with_connection(move |connection| {
            let Some(existing) = tasks_repository::get(connection, &id)? else {
                return Err(not_found());
            };
            let timestamp = now_utc();
            let completed_at = completion_timestamp(&existing, &status, &timestamp);
            tasks_repository::update_status(
                connection,
                &id,
                &status,
                completed_at.as_deref(),
                &timestamp,
            )?;
            tasks_repository::get(connection, &id)?.ok_or_else(not_found)
        })
        .await
}

pub async fn delete(database: &Database, id: String) -> Result<(), AppError> {
    database
        .with_connection(move |connection| {
            if tasks_repository::delete(connection, &id)? {
                Ok(())
            } else {
                Err(not_found())
            }
        })
        .await
}

fn completion_timestamp(task: &Task, status: &str, now: &str) -> Option<String> {
    if status == "done" {
        task.completed_at.clone().or_else(|| Some(now.to_owned()))
    } else {
        None
    }
}

fn validate_project(
    connection: &rusqlite::Connection,
    project_id: Option<&str>,
    existing_project_id: Option<&str>,
) -> Result<(), AppError> {
    let Some(project_id) = project_id else {
        return Ok(());
    };
    let Some(project) = projects_repository::get(connection, project_id)? else {
        return field_error("projectId", "Choose an existing project.");
    };
    if project.archived && existing_project_id != Some(project_id) {
        return field_error("projectId", "Archived projects cannot receive new tasks.");
    }
    Ok(())
}

fn validate_tags(connection: &rusqlite::Connection, tag_ids: &[String]) -> Result<(), AppError> {
    for tag_id in tag_ids {
        if tags_repository::get(connection, tag_id)?.is_none() {
            return field_error("tagIds", "Choose existing tags.");
        }
    }
    Ok(())
}

fn field_error(field: &str, message: &str) -> Result<(), AppError> {
    let mut fields = BTreeMap::new();
    fields.insert(field.to_owned(), message.to_owned());
    Err(AppError::validation(fields))
}

fn not_found() -> AppError {
    AppError::new("TASK_NOT_FOUND", "The task no longer exists.")
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{completion_timestamp, create, list, set_status, update};
    use crate::{
        db::{DataEnvironment, Database, StoragePaths},
        domain::tasks::{ProjectInput, TagInput, Task, TaskFilters, TaskInput},
        services::{projects, tags},
    };

    fn task(status: &str, completed_at: Option<&str>) -> Task {
        Task {
            id: "task".to_owned(),
            title: "Task".to_owned(),
            notes: None,
            due_date: None,
            due_time: None,
            priority: "medium".to_owned(),
            status: status.to_owned(),
            project: None,
            tags: vec![],
            completed_at: completed_at.map(str::to_owned),
            created_at: "created".to_owned(),
            updated_at: "updated".to_owned(),
        }
    }

    #[test]
    fn completing_records_a_timestamp_and_keeps_it_while_done() {
        assert_eq!(
            completion_timestamp(&task("todo", None), "done", "now"),
            Some("now".to_owned())
        );
        assert_eq!(
            completion_timestamp(&task("done", Some("original")), "done", "now"),
            Some("original".to_owned())
        );
    }

    #[test]
    fn reopening_clears_the_completion_timestamp() {
        assert_eq!(
            completion_timestamp(&task("done", Some("original")), "todo", "now"),
            None
        );
    }

    #[test]
    fn task_service_flow_persists_and_survives_project_and_tag_deletion() {
        tauri::async_runtime::block_on(async {
            let directory = tempdir().unwrap();
            let paths = StoragePaths::new(directory.path().to_owned(), DataEnvironment::Test);
            let database = Database::initialize(paths.clone()).unwrap();
            let project = projects::create(
                &database,
                ProjectInput {
                    name: "Work".to_owned(),
                    color: None,
                },
            )
            .await
            .unwrap();
            let tag = tags::create(
                &database,
                TagInput {
                    name: "Focus".to_owned(),
                    color: None,
                },
            )
            .await
            .unwrap();
            let created = create(
                &database,
                TaskInput {
                    title: "Prepare review".to_owned(),
                    notes: Some("Keep the draft".to_owned()),
                    due_date: Some("2026-10-01".to_owned()),
                    due_time: Some("09:30".to_owned()),
                    priority: "high".to_owned(),
                    status: "todo".to_owned(),
                    project_id: Some(project.id.clone()),
                    tag_ids: vec![tag.id.clone()],
                },
            )
            .await
            .unwrap();
            let edited = update(
                &database,
                created.id.clone(),
                TaskInput {
                    title: "Prepare final review".to_owned(),
                    notes: created.notes.clone(),
                    due_date: created.due_date.clone(),
                    due_time: None,
                    priority: "medium".to_owned(),
                    status: "in_progress".to_owned(),
                    project_id: Some(project.id.clone()),
                    tag_ids: vec![tag.id.clone()],
                },
            )
            .await
            .unwrap();
            assert_eq!(edited.due_time, None);
            assert_eq!(edited.priority, "medium");
            projects::set_archived(&database, project.id.clone(), true)
                .await
                .unwrap();
            let retained = update(
                &database,
                edited.id.clone(),
                TaskInput {
                    title: edited.title.clone(),
                    notes: edited.notes.clone(),
                    due_date: edited.due_date.clone(),
                    due_time: edited.due_time.clone(),
                    priority: edited.priority.clone(),
                    status: edited.status.clone(),
                    project_id: Some(project.id.clone()),
                    tag_ids: vec![tag.id.clone()],
                },
            )
            .await
            .unwrap();
            assert_eq!(retained.project.unwrap().id, project.id.clone());
            let archived_assignment = create(
                &database,
                TaskInput {
                    title: "New archived assignment".to_owned(),
                    notes: None,
                    due_date: None,
                    due_time: None,
                    priority: "low".to_owned(),
                    status: "todo".to_owned(),
                    project_id: Some(project.id.clone()),
                    tag_ids: vec![],
                },
            )
            .await
            .unwrap_err();
            assert!(archived_assignment
                .field_errors
                .unwrap()
                .contains_key("projectId"));
            let completed = set_status(&database, created.id.clone(), "done".to_owned())
                .await
                .unwrap();
            assert!(completed.completed_at.is_some());
            let reopened = set_status(&database, created.id.clone(), "todo".to_owned())
                .await
                .unwrap();
            assert!(reopened.completed_at.is_none());
            drop(database);

            let reopened_database = Database::initialize(paths).unwrap();
            let filtered = list(
                &reopened_database,
                TaskFilters {
                    search: Some("final".to_owned()),
                    status: Some("todo".to_owned()),
                    priority: Some("medium".to_owned()),
                    project_id: Some(project.id.clone()),
                    tag_id: Some(tag.id.clone()),
                    sort_field: Some("createdAt".to_owned()),
                    sort_direction: Some("desc".to_owned()),
                },
            )
            .await
            .unwrap();
            assert_eq!(filtered.len(), 1);
            projects::delete(&reopened_database, project.id)
                .await
                .unwrap();
            tags::delete(&reopened_database, tag.id).await.unwrap();
            let remaining = list(&reopened_database, TaskFilters::default())
                .await
                .unwrap();
            assert_eq!(remaining.len(), 1);
            assert!(remaining[0].project.is_none());
            assert!(remaining[0].tags.is_empty());
        });
    }
}
