use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    domain::goals::{progress_basis_points, Goal, GoalFilters},
    error::AppError,
};

pub struct GoalWrite<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub target_value: i64,
    pub current_value: i64,
    pub decimal_scale: u8,
    pub unit: &'a str,
    pub start_date: &'a str,
    pub end_date: Option<&'a str>,
    pub status: &'a str,
    pub completed_at: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

pub fn list(connection: &Connection, filters: &GoalFilters) -> Result<Vec<Goal>, AppError> {
    let sql = format!(
        "SELECT id, title, description, target_value, current_value, decimal_scale,
                unit, start_date, end_date, status, completed_at, created_at, updated_at
         FROM goals WHERE (?1 IS NULL OR status = ?1) ORDER BY {}",
        filters.order_by()
    );
    let mut statement = connection.prepare(&sql).map_err(AppError::from)?;
    let goals = statement
        .query_map([&filters.status], map_row)
        .map_err(AppError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    Ok(goals)
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<Goal>, AppError> {
    connection
        .query_row(
            "SELECT id, title, description, target_value, current_value, decimal_scale,
                    unit, start_date, end_date, status, completed_at, created_at, updated_at
             FROM goals WHERE id = ?1",
            [id],
            map_row,
        )
        .optional()
        .map_err(AppError::from)
}

pub fn insert(connection: &Connection, goal: &GoalWrite<'_>) -> Result<Goal, AppError> {
    connection
        .execute(
            "INSERT INTO goals (id, title, description, target_value, current_value,
         decimal_scale, unit, start_date, end_date, status, completed_at, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                goal.id,
                goal.title,
                goal.description,
                goal.target_value,
                goal.current_value,
                goal.decimal_scale,
                goal.unit,
                goal.start_date,
                goal.end_date,
                goal.status,
                goal.completed_at,
                goal.created_at,
                goal.updated_at
            ],
        )
        .map_err(AppError::from)?;
    get(connection, goal.id)?.ok_or_else(AppError::database_unavailable)
}

pub fn update(connection: &Connection, goal: &GoalWrite<'_>) -> Result<Option<Goal>, AppError> {
    let count = connection
        .execute(
            "UPDATE goals SET title=?2, description=?3, target_value=?4, current_value=?5,
         decimal_scale=?6, unit=?7, start_date=?8, end_date=?9, status=?10,
         completed_at=?11, updated_at=?12 WHERE id=?1",
            params![
                goal.id,
                goal.title,
                goal.description,
                goal.target_value,
                goal.current_value,
                goal.decimal_scale,
                goal.unit,
                goal.start_date,
                goal.end_date,
                goal.status,
                goal.completed_at,
                goal.updated_at
            ],
        )
        .map_err(AppError::from)?;
    if count == 0 {
        Ok(None)
    } else {
        get(connection, goal.id)
    }
}

pub fn update_progress(
    connection: &Connection,
    id: &str,
    value: i64,
    updated_at: &str,
) -> Result<bool, AppError> {
    connection
        .execute(
            "UPDATE goals SET current_value=?2, updated_at=?3 WHERE id=?1",
            params![id, value, updated_at],
        )
        .map(|count| count == 1)
        .map_err(AppError::from)
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
            "UPDATE goals SET status=?2, completed_at=?3, updated_at=?4 WHERE id=?1",
            params![id, status, completed_at, updated_at],
        )
        .map(|count| count == 1)
        .map_err(AppError::from)
}

pub fn delete(connection: &Connection, id: &str) -> Result<bool, AppError> {
    connection
        .execute("DELETE FROM goals WHERE id=?1", [id])
        .map(|count| count == 1)
        .map_err(AppError::from)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Goal> {
    let target: i64 = row.get(3)?;
    let current: i64 = row.get(4)?;
    let progress = progress_basis_points(current, target)
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, current))?;
    Ok(Goal {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        target_value: target.to_string(),
        current_value: current.to_string(),
        decimal_scale: row.get(5)?,
        unit: row.get(6)?,
        start_date: row.get(7)?,
        end_date: row.get(8)?,
        status: row.get(9)?,
        completed_at: row.get(10)?,
        progress_basis_points: progress,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    #[test]
    fn crud_filters_sorting_and_constraints_use_real_sqlite() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrations::initialize_new_database(&connection).unwrap();
        migrations::apply_pending(&mut connection, 0).unwrap();
        let write = GoalWrite {
            id: "one",
            title: "Learn",
            description: None,
            target_value: 1000,
            current_value: 250,
            decimal_scale: 1,
            unit: "hours",
            start_date: "2026-01-01",
            end_date: Some("2026-12-31"),
            status: "active",
            completed_at: None,
            created_at: "created",
            updated_at: "updated",
        };
        assert_eq!(
            insert(&connection, &write).unwrap().progress_basis_points,
            "2500"
        );
        let filters = GoalFilters {
            status: Some("active".into()),
            sort_field: Some("progress".into()),
            sort_direction: Some("desc".into()),
        }
        .normalize()
        .unwrap();
        assert_eq!(list(&connection, &filters).unwrap().len(), 1);
        assert!(update_progress(&connection, "one", 1200, "later").unwrap());
        assert_eq!(
            get(&connection, "one")
                .unwrap()
                .unwrap()
                .progress_basis_points,
            "12000"
        );
        assert!(connection.execute("INSERT INTO goals (id,title,target_value,current_value,decimal_scale,unit,start_date,status,created_at,updated_at) VALUES ('bad','Bad',0,0,0,'x','2026-01-01','active','now','now')",[]).is_err());
        assert!(delete(&connection, "one").unwrap());
    }
}
