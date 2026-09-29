use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    domain::tasks::{Project, Tag, Task, TaskFilters},
    error::AppError,
};

#[derive(Debug, Clone)]
pub struct TaskWrite<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub notes: Option<&'a str>,
    pub due_date: Option<&'a str>,
    pub due_time: Option<&'a str>,
    pub priority: &'a str,
    pub status: &'a str,
    pub project_id: Option<&'a str>,
    pub completed_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
    pub tag_ids: &'a [String],
}

pub fn list(connection: &Connection, filters: &TaskFilters) -> Result<Vec<Task>, AppError> {
    let sql = format!(
        "SELECT t.id, t.title, t.notes, t.due_date, t.due_time, t.priority,
                t.status, t.completed_at, t.created_at, t.updated_at,
                p.id, p.name, p.color, p.archived, p.created_at, p.updated_at
         FROM tasks t
         LEFT JOIN projects p ON p.id = t.project_id
         WHERE (?1 IS NULL OR instr(lower(t.title), lower(?1)) > 0
                OR instr(lower(COALESCE(t.notes, '')), lower(?1)) > 0)
           AND (?2 IS NULL OR t.status = ?2)
           AND (?3 IS NULL OR t.priority = ?3)
           AND (?4 IS NULL OR t.project_id = ?4)
           AND (?5 IS NULL OR EXISTS (
                SELECT 1 FROM task_tags filter_tags
                WHERE filter_tags.task_id = t.id AND filter_tags.tag_id = ?5
           ))
         ORDER BY {}",
        filters.order_by()
    );
    let mut statement = connection.prepare(&sql).map_err(AppError::from)?;
    let rows = statement
        .query_map(
            params![
                filters.search,
                filters.status,
                filters.priority,
                filters.project_id,
                filters.tag_id,
            ],
            map_task_row,
        )
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    rows.into_iter()
        .map(|task| hydrate_tags(connection, task))
        .collect()
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<Task>, AppError> {
    let task = connection
        .query_row(
            "SELECT t.id, t.title, t.notes, t.due_date, t.due_time, t.priority,
                    t.status, t.completed_at, t.created_at, t.updated_at,
                    p.id, p.name, p.color, p.archived, p.created_at, p.updated_at
             FROM tasks t
             LEFT JOIN projects p ON p.id = t.project_id
             WHERE t.id = ?1",
            [id],
            map_task_row,
        )
        .optional()
        .map_err(AppError::from)?;
    task.map(|task| hydrate_tags(connection, task)).transpose()
}

pub fn insert(connection: &mut Connection, task: &TaskWrite<'_>) -> Result<Task, AppError> {
    let transaction = connection.transaction().map_err(AppError::from)?;
    transaction
        .execute(
            "INSERT INTO tasks (
                id, title, notes, due_date, due_time, priority, status,
                project_id, completed_at, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task.id,
                task.title,
                task.notes,
                task.due_date,
                task.due_time,
                task.priority,
                task.status,
                task.project_id,
                task.completed_at,
                task.created_at,
                task.updated_at,
            ],
        )
        .map_err(AppError::from)?;
    replace_tags(&transaction, task.id, task.tag_ids)?;
    transaction.commit().map_err(AppError::from)?;
    get(connection, task.id)?.ok_or_else(AppError::database_unavailable)
}

pub fn update(connection: &mut Connection, task: &TaskWrite<'_>) -> Result<Option<Task>, AppError> {
    let transaction = connection.transaction().map_err(AppError::from)?;
    let updated = transaction
        .execute(
            "UPDATE tasks SET
                title = ?2, notes = ?3, due_date = ?4, due_time = ?5,
                priority = ?6, status = ?7, project_id = ?8,
                completed_at = ?9, updated_at = ?10
             WHERE id = ?1",
            params![
                task.id,
                task.title,
                task.notes,
                task.due_date,
                task.due_time,
                task.priority,
                task.status,
                task.project_id,
                task.completed_at,
                task.updated_at,
            ],
        )
        .map_err(AppError::from)?;
    if updated == 0 {
        return Ok(None);
    }
    replace_tags(&transaction, task.id, task.tag_ids)?;
    transaction.commit().map_err(AppError::from)?;
    get(connection, task.id)
}

pub fn update_status(
    connection: &Connection,
    id: &str,
    status: &str,
    completed_at: Option<&str>,
    updated_at: &str,
) -> Result<bool, AppError> {
    connection
        .execute(
            "UPDATE tasks SET status = ?2, completed_at = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, status, completed_at, updated_at],
        )
        .map(|count| count == 1)
        .map_err(AppError::from)
}

pub fn delete(connection: &Connection, id: &str) -> Result<bool, AppError> {
    connection
        .execute("DELETE FROM tasks WHERE id = ?1", [id])
        .map(|count| count == 1)
        .map_err(AppError::from)
}

fn replace_tags(
    connection: &Connection,
    task_id: &str,
    tag_ids: &[String],
) -> Result<(), AppError> {
    connection
        .execute("DELETE FROM task_tags WHERE task_id = ?1", [task_id])
        .map_err(AppError::from)?;
    for tag_id in tag_ids {
        connection
            .execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![task_id, tag_id],
            )
            .map_err(AppError::from)?;
    }
    Ok(())
}

fn hydrate_tags(connection: &Connection, mut task: Task) -> Result<Task, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT g.id, g.name, g.color, g.created_at, g.updated_at
             FROM tags g
             INNER JOIN task_tags tt ON tt.tag_id = g.id
             WHERE tt.task_id = ?1
             ORDER BY g.name COLLATE NOCASE ASC",
        )
        .map_err(AppError::from)?;
    task.tags = statement
        .query_map([&task.id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(task)
}

fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let project_id: Option<String> = row.get(10)?;
    let project = if let Some(id) = project_id {
        Some(Project {
            id,
            name: row.get(11)?,
            color: row.get(12)?,
            archived: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    } else {
        None
    };
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        notes: row.get(2)?,
        due_date: row.get(3)?,
        due_time: row.get(4)?,
        priority: row.get(5)?,
        status: row.get(6)?,
        completed_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        project,
        tags: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{get, insert, list, update, TaskWrite};
    use crate::{
        db::{migrations, projects_repository, tags_repository},
        domain::tasks::{Project, Tag, TaskFilters},
    };

    fn setup() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrations::initialize_new_database(&connection).unwrap();
        migrations::apply_pending(&mut connection, 0).unwrap();
        connection
    }

    fn project(id: &str) -> Project {
        Project {
            id: id.to_owned(),
            name: format!("Project {id}"),
            color: Some("#17735a".to_owned()),
            archived: false,
            created_at: "2026-09-30T00:00:00.000Z".to_owned(),
            updated_at: "2026-09-30T00:00:00.000Z".to_owned(),
        }
    }

    fn tag(id: &str, name: &str) -> Tag {
        Tag {
            id: id.to_owned(),
            name: name.to_owned(),
            color: None,
            created_at: "2026-09-30T00:00:00.000Z".to_owned(),
            updated_at: "2026-09-30T00:00:00.000Z".to_owned(),
        }
    }

    fn write<'a>(id: &'a str, project_id: Option<&'a str>, tag_ids: &'a [String]) -> TaskWrite<'a> {
        TaskWrite {
            id,
            title: "Write report",
            notes: Some("Quarterly notes"),
            due_date: Some("2026-10-02"),
            due_time: Some("09:30"),
            priority: "high",
            status: "todo",
            project_id,
            completed_at: None,
            created_at: "2026-09-30T00:00:00.000Z",
            updated_at: "2026-09-30T00:00:00.000Z",
            tag_ids,
        }
    }

    #[test]
    fn task_project_tag_crud_and_combined_filters_work() {
        let mut connection = setup();
        let work = project("work");
        projects_repository::insert(&connection, &work).unwrap();
        let focus = tag("focus", "Focus");
        let admin = tag("admin", "Admin");
        tags_repository::insert(&connection, &focus).unwrap();
        tags_repository::insert(&connection, &admin).unwrap();
        let both = vec![focus.id.clone(), admin.id.clone()];
        insert(&mut connection, &write("task-1", Some("work"), &both)).unwrap();

        let fetched = get(&connection, "task-1").unwrap().unwrap();
        assert_eq!(fetched.project.unwrap().id, "work");
        assert_eq!(fetched.tags.len(), 2);

        let one_tag = vec![focus.id.clone()];
        let mut changed = write("task-1", Some("work"), &one_tag);
        changed.title = "Write final report";
        changed.status = "done";
        changed.completed_at = Some("2026-09-30T01:00:00.000Z");
        let changed = update(&mut connection, &changed).unwrap().unwrap();
        assert_eq!(changed.title, "Write final report");
        assert_eq!(changed.tags.len(), 1);

        let filters = TaskFilters {
            search: Some("final".to_owned()),
            status: Some("done".to_owned()),
            priority: Some("high".to_owned()),
            project_id: Some("work".to_owned()),
            tag_id: Some("focus".to_owned()),
            sort_field: Some("priority".to_owned()),
            sort_direction: Some("desc".to_owned()),
        }
        .normalize()
        .unwrap();
        assert_eq!(list(&connection, &filters).unwrap().len(), 1);

        projects_repository::delete(&connection, "work").unwrap();
        let after_project_delete = get(&connection, "task-1").unwrap().unwrap();
        assert!(after_project_delete.project.is_none());

        tags_repository::delete(&connection, "focus").unwrap();
        let after_tag_delete = get(&connection, "task-1").unwrap().unwrap();
        assert!(after_tag_delete.tags.is_empty());
        assert!(super::delete(&connection, "task-1").unwrap());
        assert!(get(&connection, "task-1").unwrap().is_none());
    }

    #[test]
    fn task_insert_rolls_back_when_a_tag_relationship_fails() {
        let mut connection = setup();
        let missing = vec!["missing-tag".to_owned()];
        let error = insert(&mut connection, &write("task-rollback", None, &missing)).unwrap_err();
        assert!(matches!(
            error.code.as_str(),
            "INTERNAL_ERROR" | "WRITE_FAILED"
        ));
        assert!(get(&connection, "task-rollback").unwrap().is_none());
    }

    #[test]
    fn database_constraints_reject_invalid_task_states_and_duplicate_tags() {
        let connection = setup();
        let first = tag("tag-one", "Focus");
        tags_repository::insert(&connection, &first).unwrap();
        let duplicate = tag("tag-two", "focus");
        assert!(tags_repository::insert(&connection, &duplicate).is_err());

        let invalid = connection.execute(
            "INSERT INTO tasks (
                id, title, due_time, priority, status, completed_at, created_at, updated_at
             ) VALUES ('invalid', 'Invalid', '09:00', 'medium', 'done', NULL, 'now', 'now')",
            [],
        );
        assert!(invalid.is_err());
    }
}
