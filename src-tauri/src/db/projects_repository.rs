use rusqlite::{params, Connection, OptionalExtension};

use crate::{domain::tasks::Project, error::AppError};

pub fn list(connection: &Connection) -> Result<Vec<Project>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, color, archived, created_at, updated_at
             FROM projects ORDER BY archived ASC, name COLLATE NOCASE ASC, id ASC",
        )
        .map_err(AppError::from)?;
    let projects = statement
        .query_map([], map_project)
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(projects)
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<Project>, AppError> {
    connection
        .query_row(
            "SELECT id, name, color, archived, created_at, updated_at
             FROM projects WHERE id = ?1",
            [id],
            map_project,
        )
        .optional()
        .map_err(AppError::from)
}

pub fn insert(connection: &Connection, project: &Project) -> Result<(), AppError> {
    connection
        .execute(
            "INSERT INTO projects (id, name, color, archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project.id,
                project.name,
                project.color,
                project.archived,
                project.created_at,
                project.updated_at,
            ],
        )
        .map_err(AppError::from)?;
    Ok(())
}

pub fn update(connection: &Connection, project: &Project) -> Result<bool, AppError> {
    connection
        .execute(
            "UPDATE projects SET name = ?2, color = ?3, archived = ?4, updated_at = ?5
             WHERE id = ?1",
            params![
                project.id,
                project.name,
                project.color,
                project.archived,
                project.updated_at,
            ],
        )
        .map(|count| count == 1)
        .map_err(AppError::from)
}

pub fn delete(connection: &Connection, id: &str) -> Result<bool, AppError> {
    connection
        .execute("DELETE FROM projects WHERE id = ?1", [id])
        .map(|count| count == 1)
        .map_err(AppError::from)
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        archived: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{delete, get, insert, list, update};
    use crate::{db::migrations, domain::tasks::Project};

    #[test]
    fn project_crud_and_archive_round_trip() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrations::initialize_new_database(&connection).unwrap();
        migrations::apply_pending(&mut connection, 0).unwrap();
        let mut project = Project {
            id: "project".to_owned(),
            name: "Work".to_owned(),
            color: None,
            archived: false,
            created_at: "created".to_owned(),
            updated_at: "created".to_owned(),
        };
        insert(&connection, &project).unwrap();
        assert_eq!(list(&connection).unwrap().len(), 1);
        project.name = "Deep work".to_owned();
        project.archived = true;
        project.updated_at = "updated".to_owned();
        assert!(update(&connection, &project).unwrap());
        let saved = get(&connection, "project").unwrap().unwrap();
        assert_eq!(saved.name, "Deep work");
        assert!(saved.archived);
        assert!(delete(&connection, "project").unwrap());
        assert!(get(&connection, "project").unwrap().is_none());
    }
}
