use rusqlite::{params, Connection, OptionalExtension};

use crate::{domain::tasks::Tag, error::AppError};

pub fn list(connection: &Connection) -> Result<Vec<Tag>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, color, created_at, updated_at
             FROM tags ORDER BY name COLLATE NOCASE ASC, id ASC",
        )
        .map_err(AppError::from)?;
    let tags = statement
        .query_map([], map_tag)
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(tags)
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<Tag>, AppError> {
    connection
        .query_row(
            "SELECT id, name, color, created_at, updated_at FROM tags WHERE id = ?1",
            [id],
            map_tag,
        )
        .optional()
        .map_err(AppError::from)
}

pub fn name_exists(
    connection: &Connection,
    name: &str,
    excluding_id: Option<&str>,
) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM tags
                WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id <> ?2)
             )",
            params![name, excluding_id],
            |row| row.get(0),
        )
        .map_err(AppError::from)
}

pub fn insert(connection: &Connection, tag: &Tag) -> Result<(), AppError> {
    connection
        .execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tag.id, tag.name, tag.color, tag.created_at, tag.updated_at],
        )
        .map_err(AppError::from)?;
    Ok(())
}

pub fn update(connection: &Connection, tag: &Tag) -> Result<bool, AppError> {
    connection
        .execute(
            "UPDATE tags SET name = ?2, color = ?3, updated_at = ?4 WHERE id = ?1",
            params![tag.id, tag.name, tag.color, tag.updated_at],
        )
        .map(|count| count == 1)
        .map_err(AppError::from)
}

pub fn delete(connection: &Connection, id: &str) -> Result<bool, AppError> {
    connection
        .execute("DELETE FROM tags WHERE id = ?1", [id])
        .map(|count| count == 1)
        .map_err(AppError::from)
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{delete, get, insert, list, update};
    use crate::{db::migrations, domain::tasks::Tag};

    #[test]
    fn tag_crud_round_trip() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrations::initialize_new_database(&connection).unwrap();
        migrations::apply_pending(&mut connection, 0).unwrap();
        let mut tag = Tag {
            id: "tag".to_owned(),
            name: "Focus".to_owned(),
            color: None,
            created_at: "created".to_owned(),
            updated_at: "created".to_owned(),
        };
        insert(&connection, &tag).unwrap();
        assert_eq!(list(&connection).unwrap().len(), 1);
        tag.name = "Important".to_owned();
        tag.updated_at = "updated".to_owned();
        assert!(update(&connection, &tag).unwrap());
        assert_eq!(get(&connection, "tag").unwrap().unwrap().name, "Important");
        assert!(delete(&connection, "tag").unwrap());
        assert!(get(&connection, "tag").unwrap().is_none());
    }
}
