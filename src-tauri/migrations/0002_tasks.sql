CREATE TABLE projects (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    color TEXT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0)
);

CREATE TABLE tags (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    name TEXT NOT NULL COLLATE NOCASE CHECK (length(trim(name)) > 0),
    color TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0),
    UNIQUE (name)
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    notes TEXT,
    due_date TEXT CHECK (
        due_date IS NULL
        OR (
            length(due_date) = 10
            AND substr(due_date, 5, 1) = '-'
            AND substr(due_date, 8, 1) = '-'
        )
    ),
    due_time TEXT CHECK (
        due_time IS NULL
        OR (
            length(due_time) = 5
            AND substr(due_time, 3, 1) = ':'
        )
    ),
    priority TEXT NOT NULL CHECK (priority IN ('low', 'medium', 'high')),
    status TEXT NOT NULL CHECK (status IN ('todo', 'in_progress', 'done')),
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    completed_at TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0),
    CHECK (due_time IS NULL OR due_date IS NOT NULL),
    CHECK (
        (status = 'done' AND completed_at IS NOT NULL)
        OR (status <> 'done' AND completed_at IS NULL)
    )
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

CREATE INDEX tasks_status_due_date_idx ON tasks(status, due_date);
CREATE INDEX tasks_project_id_idx ON tasks(project_id);
CREATE INDEX task_tags_tag_task_idx ON task_tags(tag_id, task_id);
