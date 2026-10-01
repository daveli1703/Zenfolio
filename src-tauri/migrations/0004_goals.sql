CREATE TABLE goals (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT,
    target_value INTEGER NOT NULL CHECK (target_value > 0),
    current_value INTEGER NOT NULL CHECK (current_value >= 0),
    decimal_scale INTEGER NOT NULL CHECK (decimal_scale BETWEEN 0 AND 3),
    unit TEXT NOT NULL CHECK (length(trim(unit)) > 0),
    start_date TEXT NOT NULL CHECK (length(start_date)=10 AND substr(start_date,5,1)='-' AND substr(start_date,8,1)='-'),
    end_date TEXT CHECK (end_date IS NULL OR (length(end_date)=10 AND substr(end_date,5,1)='-' AND substr(end_date,8,1)='-')),
    status TEXT NOT NULL CHECK (status IN ('active','completed','paused','abandoned')),
    completed_at TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at)>0),
    updated_at TEXT NOT NULL CHECK (length(updated_at)>0),
    CHECK (end_date IS NULL OR end_date >= start_date),
    CHECK ((status='completed' AND completed_at IS NOT NULL) OR (status<>'completed' AND completed_at IS NULL))
);

CREATE INDEX goals_status_end_date_idx ON goals(status, end_date);
