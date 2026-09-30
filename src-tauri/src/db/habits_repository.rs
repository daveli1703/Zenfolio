use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

use crate::{
    domain::habits::{Habit, HabitDeleteImpact, HabitEntry, HabitRule},
    error::AppError,
};

pub struct HabitWrite<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub target_type: &'a str,
    pub unit: Option<&'a str>,
    pub color: &'a str,
    pub start_date: &'a str,
    pub archive_date: Option<&'a str>,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

pub struct RuleWrite<'a> {
    pub id: &'a str,
    pub habit_id: &'a str,
    pub effective_date: &'a str,
    pub target: i64,
    pub weekday_mask: i64,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

pub fn list(connection: &Connection) -> Result<Vec<Habit>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id, name, description, target_type, unit, color, start_date, archive_date, created_at, updated_at
         FROM habits ORDER BY archive_date IS NOT NULL, name COLLATE NOCASE, id",
    )?;
    let rows = statement.query_map([], map_habit)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(connection: &Connection, id: &str) -> Result<Option<Habit>, AppError> {
    connection.query_row(
        "SELECT id, name, description, target_type, unit, color, start_date, archive_date, created_at, updated_at FROM habits WHERE id=?1",
        [id], map_habit,
    ).optional().map_err(AppError::from)
}

pub fn rules(connection: &Connection, habit_id: &str) -> Result<Vec<HabitRule>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id, habit_id, effective_date, target, weekday_mask, created_at, updated_at FROM habit_rules WHERE habit_id=?1 ORDER BY effective_date, id",
    )?;
    let rows = statement.query_map([habit_id], |row| {
        Ok(HabitRule {
            id: row.get(0)?,
            habit_id: row.get(1)?,
            effective_date: row.get(2)?,
            target: row.get(3)?,
            weekday_mask: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn entries(
    connection: &Connection,
    habit_id: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<HabitEntry>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id, habit_id, date, value, notes, created_at, updated_at FROM habit_entries
         WHERE habit_id=?1 AND (?2 IS NULL OR date>=?2) AND (?3 IS NULL OR date<=?3) ORDER BY date",
    )?;
    let rows = statement.query_map(params![habit_id, from, to], |row| {
        Ok(HabitEntry {
            id: row.get(0)?,
            habit_id: row.get(1)?,
            date: row.get(2)?,
            value: row.get(3)?,
            notes: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn has_entries(connection: &Connection, habit_id: &str) -> Result<bool, AppError> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM habit_entries WHERE habit_id=?1)",
        [habit_id],
        |row| row.get::<_, i64>(0),
    )? != 0)
}

pub fn insert_with_rule(
    connection: &mut Connection,
    habit: &HabitWrite<'_>,
    rule: &RuleWrite<'_>,
) -> Result<(), AppError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO habits (id,name,description,target_type,unit,color,start_date,archive_date,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![habit.id,habit.name,habit.description,habit.target_type,habit.unit,habit.color,habit.start_date,habit.archive_date,habit.created_at,habit.updated_at],
    )?;
    insert_rule(&transaction, rule)?;
    transaction.commit()?;
    Ok(())
}

pub fn update_profile(connection: &Connection, habit: &HabitWrite<'_>) -> Result<bool, AppError> {
    Ok(connection.execute(
        "UPDATE habits SET name=?2,description=?3,target_type=?4,unit=?5,color=?6,start_date=?7,updated_at=?8 WHERE id=?1",
        params![habit.id,habit.name,habit.description,habit.target_type,habit.unit,habit.color,habit.start_date,habit.updated_at],
    )? > 0)
}

pub fn update_initial_rule_date(
    connection: &Connection,
    habit_id: &str,
    old_date: &str,
    new_date: &str,
    updated_at: &str,
) -> Result<(), AppError> {
    connection.execute("UPDATE habit_rules SET effective_date=?3,updated_at=?4 WHERE habit_id=?1 AND effective_date=?2", params![habit_id,old_date,new_date,updated_at])?;
    Ok(())
}

pub fn normalize_boolean_rules(
    connection: &Connection,
    habit_id: &str,
    updated_at: &str,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE habit_rules SET target=1, updated_at=?2 WHERE habit_id=?1",
        params![habit_id, updated_at],
    )?;
    Ok(())
}

pub fn upsert_rule(connection: &Connection, rule: &RuleWrite<'_>) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO habit_rules (id,habit_id,effective_date,target,weekday_mask,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(habit_id,effective_date) DO UPDATE SET target=excluded.target,weekday_mask=excluded.weekday_mask,updated_at=excluded.updated_at",
        params![rule.id,rule.habit_id,rule.effective_date,rule.target,rule.weekday_mask,rule.created_at,rule.updated_at],
    )?;
    Ok(())
}

pub fn archive(
    connection: &Connection,
    id: &str,
    archive_date: &str,
    updated_at: &str,
) -> Result<bool, AppError> {
    Ok(connection.execute(
        "UPDATE habits SET archive_date=?2,updated_at=?3 WHERE id=?1",
        params![id, archive_date, updated_at],
    )? > 0)
}

pub fn upsert_entry(connection: &Connection, entry: &HabitEntry) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO habit_entries (id,habit_id,date,value,notes,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(habit_id,date) DO UPDATE SET value=excluded.value,notes=excluded.notes,updated_at=excluded.updated_at",
        params![entry.id,entry.habit_id,entry.date,entry.value,entry.notes,entry.created_at,entry.updated_at],
    )?;
    Ok(())
}

pub fn delete_impact(
    connection: &Connection,
    id: &str,
) -> Result<Option<HabitDeleteImpact>, AppError> {
    if get(connection, id)?.is_none() {
        return Ok(None);
    }
    Ok(Some(HabitDeleteImpact {
        rule_count: connection.query_row(
            "SELECT count(*) FROM habit_rules WHERE habit_id=?1",
            [id],
            |row| row.get(0),
        )?,
        entry_count: connection.query_row(
            "SELECT count(*) FROM habit_entries WHERE habit_id=?1",
            [id],
            |row| row.get(0),
        )?,
    }))
}

pub fn delete(connection: &Connection, id: &str) -> Result<bool, AppError> {
    Ok(connection.execute("DELETE FROM habits WHERE id=?1", [id])? > 0)
}

fn insert_rule(connection: &Connection, rule: &RuleWrite<'_>) -> Result<(), AppError> {
    connection.execute("INSERT INTO habit_rules (id,habit_id,effective_date,target,weekday_mask,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![rule.id,rule.habit_id,rule.effective_date,rule.target,rule.weekday_mask,rule.created_at,rule.updated_at])?;
    Ok(())
}

fn map_habit(row: &rusqlite::Row<'_>) -> rusqlite::Result<Habit> {
    Ok(Habit {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        target_type: row.get(3)?,
        unit: row.get(4)?,
        color: row.get(5)?,
        start_date: row.get(6)?,
        archive_date: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}
