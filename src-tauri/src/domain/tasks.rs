use std::collections::{BTreeMap, HashSet};

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const PRIORITIES: [&str; 3] = ["low", "medium", "high"];
pub const STATUSES: [&str; 3] = ["todo", "in_progress", "done"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TagInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub priority: String,
    pub status: String,
    pub project: Option<Project>,
    pub tags: Vec<Tag>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub due_time: Option<String>,
    pub priority: String,
    pub status: String,
    pub project_id: Option<String>,
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_id: Option<String>,
    pub tag_id: Option<String>,
    pub sort_field: Option<String>,
    pub sort_direction: Option<String>,
}

impl ProjectInput {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.name = self.name.trim().to_owned();
        self.color = normalize_optional(self.color);
        let mut errors = BTreeMap::new();
        if self.name.is_empty() {
            errors.insert("name".to_owned(), "Project name is required.".to_owned());
        }
        validate_color(&self.color, &mut errors);
        validation_result(errors)?;
        Ok(self)
    }
}

impl TagInput {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.name = self.name.trim().to_owned();
        self.color = normalize_optional(self.color);
        let mut errors = BTreeMap::new();
        if self.name.is_empty() {
            errors.insert("name".to_owned(), "Tag name is required.".to_owned());
        }
        validate_color(&self.color, &mut errors);
        validation_result(errors)?;
        Ok(self)
    }
}

impl TaskInput {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.title = self.title.trim().to_owned();
        self.notes = normalize_optional(self.notes);
        self.due_date = normalize_optional(self.due_date);
        self.due_time = normalize_optional(self.due_time);
        self.project_id = normalize_optional(self.project_id);
        self.tag_ids = unique_nonblank(self.tag_ids);

        let mut errors = BTreeMap::new();
        if self.title.is_empty() {
            errors.insert("title".to_owned(), "Task title is required.".to_owned());
        }
        if !PRIORITIES.contains(&self.priority.as_str()) {
            errors.insert("priority".to_owned(), "Choose a valid priority.".to_owned());
        }
        if !STATUSES.contains(&self.status.as_str()) {
            errors.insert("status".to_owned(), "Choose a valid status.".to_owned());
        }
        if let Some(date) = &self.due_date {
            if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
                errors.insert("dueDate".to_owned(), "Use a valid due date.".to_owned());
            }
        }
        if let Some(time) = &self.due_time {
            if self.due_date.is_none() {
                errors.insert(
                    "dueTime".to_owned(),
                    "Choose a due date before adding a due time.".to_owned(),
                );
            } else if NaiveTime::parse_from_str(time, "%H:%M").is_err() {
                errors.insert("dueTime".to_owned(), "Use a valid due time.".to_owned());
            }
        }
        validation_result(errors)?;
        Ok(self)
    }
}

impl TaskFilters {
    pub fn normalize(mut self) -> Result<Self, AppError> {
        self.search = normalize_optional(self.search);
        self.status = normalize_optional(self.status);
        self.priority = normalize_optional(self.priority);
        self.project_id = normalize_optional(self.project_id);
        self.tag_id = normalize_optional(self.tag_id);
        self.sort_field = normalize_optional(self.sort_field).or(Some("dueDate".to_owned()));
        self.sort_direction = normalize_optional(self.sort_direction).or(Some("asc".to_owned()));

        let mut errors = BTreeMap::new();
        if let Some(status) = &self.status {
            if !STATUSES.contains(&status.as_str()) {
                errors.insert(
                    "status".to_owned(),
                    "Choose a valid status filter.".to_owned(),
                );
            }
        }
        if let Some(priority) = &self.priority {
            if !PRIORITIES.contains(&priority.as_str()) {
                errors.insert(
                    "priority".to_owned(),
                    "Choose a valid priority filter.".to_owned(),
                );
            }
        }
        if !matches!(
            self.sort_field.as_deref(),
            Some("dueDate" | "createdAt" | "priority")
        ) {
            errors.insert(
                "sortField".to_owned(),
                "Choose a valid sort field.".to_owned(),
            );
        }
        if !matches!(self.sort_direction.as_deref(), Some("asc" | "desc")) {
            errors.insert(
                "sortDirection".to_owned(),
                "Choose a valid sort direction.".to_owned(),
            );
        }
        validation_result(errors)?;
        Ok(self)
    }

    pub fn order_by(&self) -> &'static str {
        match (self.sort_field.as_deref(), self.sort_direction.as_deref()) {
            (Some("createdAt"), Some("asc")) => "t.created_at ASC, t.id ASC",
            (Some("createdAt"), Some("desc")) => "t.created_at DESC, t.id DESC",
            (Some("priority"), Some("asc")) => {
                "CASE t.priority WHEN 'low' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END ASC, t.created_at DESC"
            }
            (Some("priority"), Some("desc")) => {
                "CASE t.priority WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END ASC, t.created_at DESC"
            }
            (Some("dueDate"), Some("desc")) => {
                "t.due_date IS NULL ASC, t.due_date DESC, t.due_time IS NULL ASC, t.due_time DESC, t.created_at DESC"
            }
            _ => {
                "t.due_date IS NULL ASC, t.due_date ASC, t.due_time IS NULL ASC, t.due_time ASC, t.created_at DESC"
            }
        }
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    })
}

fn unique_nonblank(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .collect()
}

fn validate_color(color: &Option<String>, errors: &mut BTreeMap<String, String>) {
    if let Some(color) = color {
        let valid = color.len() == 7
            && color.starts_with('#')
            && color[1..]
                .chars()
                .all(|character| character.is_ascii_hexdigit());
        if !valid {
            errors.insert("color".to_owned(), "Use a six-digit hex color.".to_owned());
        }
    }
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
    use super::{TaskFilters, TaskInput};

    fn valid_task() -> TaskInput {
        TaskInput {
            title: "Write report".to_owned(),
            notes: None,
            due_date: None,
            due_time: None,
            priority: "medium".to_owned(),
            status: "todo".to_owned(),
            project_id: None,
            tag_ids: vec![],
        }
    }

    #[test]
    fn rejects_blank_title_and_time_without_date() {
        let mut input = valid_task();
        input.title = "  ".to_owned();
        input.due_time = Some("09:30".to_owned());
        let error = input.normalize().expect_err("invalid task");
        let fields = error.field_errors.unwrap();
        assert!(fields.contains_key("title"));
        assert!(fields.contains_key("dueTime"));
    }

    #[test]
    fn rejects_invalid_status_and_priority() {
        let mut input = valid_task();
        input.status = "later".to_owned();
        input.priority = "urgent".to_owned();
        let fields = input.normalize().unwrap_err().field_errors.unwrap();
        assert!(fields.contains_key("status"));
        assert!(fields.contains_key("priority"));
    }

    #[test]
    fn sort_fields_and_directions_are_allowlisted() {
        let error = TaskFilters {
            sort_field: Some("title; DROP TABLE tasks".to_owned()),
            sort_direction: Some("sideways".to_owned()),
            ..TaskFilters::default()
        }
        .normalize()
        .unwrap_err();
        let fields = error.field_errors.unwrap();
        assert!(fields.contains_key("sortField"));
        assert!(fields.contains_key("sortDirection"));
    }
}
