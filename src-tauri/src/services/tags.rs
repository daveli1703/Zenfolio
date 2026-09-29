use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{
    db::{migrations::now_utc, tags_repository, Database},
    domain::tasks::{Tag, TagInput},
    error::AppError,
};

pub async fn list(database: &Database) -> Result<Vec<Tag>, AppError> {
    database
        .with_connection(|connection| tags_repository::list(connection))
        .await
}

pub async fn create(database: &Database, input: TagInput) -> Result<Tag, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            ensure_unique(connection, &input.name, None)?;
            let timestamp = now_utc();
            let tag = Tag {
                id: Uuid::new_v4().to_string(),
                name: input.name,
                color: input.color,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            };
            tags_repository::insert(connection, &tag)?;
            Ok(tag)
        })
        .await
}

pub async fn update(database: &Database, id: String, input: TagInput) -> Result<Tag, AppError> {
    let input = input.normalize()?;
    database
        .with_connection(move |connection| {
            let Some(existing) = tags_repository::get(connection, &id)? else {
                return Err(not_found());
            };
            ensure_unique(connection, &input.name, Some(&id))?;
            let tag = Tag {
                id,
                name: input.name,
                color: input.color,
                created_at: existing.created_at,
                updated_at: now_utc(),
            };
            tags_repository::update(connection, &tag)?;
            Ok(tag)
        })
        .await
}

pub async fn delete(database: &Database, id: String) -> Result<(), AppError> {
    database
        .with_connection(move |connection| {
            if tags_repository::delete(connection, &id)? {
                Ok(())
            } else {
                Err(not_found())
            }
        })
        .await
}

fn ensure_unique(
    connection: &rusqlite::Connection,
    name: &str,
    excluding_id: Option<&str>,
) -> Result<(), AppError> {
    if tags_repository::name_exists(connection, name, excluding_id)? {
        let mut fields = BTreeMap::new();
        fields.insert(
            "name".to_owned(),
            "A tag with this name already exists.".to_owned(),
        );
        Err(AppError::validation(fields))
    } else {
        Ok(())
    }
}

fn not_found() -> AppError {
    AppError::new("TAG_NOT_FOUND", "The tag no longer exists.")
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::ensure_unique;
    use crate::{
        db::{migrations, tags_repository},
        domain::tasks::Tag,
    };

    #[test]
    fn tag_uniqueness_is_case_insensitive() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrations::initialize_new_database(&connection).unwrap();
        migrations::apply_pending(&mut connection, 0).unwrap();
        tags_repository::insert(
            &connection,
            &Tag {
                id: "one".to_owned(),
                name: "Focus".to_owned(),
                color: None,
                created_at: "created".to_owned(),
                updated_at: "updated".to_owned(),
            },
        )
        .unwrap();

        let error = ensure_unique(&connection, "focus", None).unwrap_err();
        assert_eq!(error.code, "VALIDATION_ERROR");
        assert!(error.field_errors.unwrap().contains_key("name"));
    }
}
