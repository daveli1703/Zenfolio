CREATE TABLE habits (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description TEXT,
    target_type TEXT NOT NULL CHECK (target_type IN ('boolean', 'count', 'duration')),
    unit TEXT,
    color TEXT NOT NULL CHECK (length(trim(color)) > 0),
    start_date TEXT NOT NULL CHECK (
        length(start_date) = 10
        AND substr(start_date, 5, 1) = '-'
        AND substr(start_date, 8, 1) = '-'
    ),
    archive_date TEXT CHECK (
        archive_date IS NULL
        OR (
            length(archive_date) = 10
            AND substr(archive_date, 5, 1) = '-'
            AND substr(archive_date, 8, 1) = '-'
        )
    ),
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0),
    CHECK (
        (target_type = 'boolean' AND unit IS NULL)
        OR (target_type = 'count' AND unit IS NOT NULL AND length(trim(unit)) > 0)
        OR (target_type = 'duration' AND unit IS NULL)
    ),
    CHECK (archive_date IS NULL OR archive_date > start_date)
);

CREATE TABLE habit_rules (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    habit_id TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    effective_date TEXT NOT NULL CHECK (
        length(effective_date) = 10
        AND substr(effective_date, 5, 1) = '-'
        AND substr(effective_date, 8, 1) = '-'
    ),
    target INTEGER NOT NULL CHECK (target > 0),
    weekday_mask INTEGER NOT NULL CHECK (weekday_mask BETWEEN 1 AND 127),
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0),
    UNIQUE (habit_id, effective_date)
);

CREATE TABLE habit_entries (
    id TEXT PRIMARY KEY CHECK (length(trim(id)) > 0),
    habit_id TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
    date TEXT NOT NULL CHECK (
        length(date) = 10
        AND substr(date, 5, 1) = '-'
        AND substr(date, 8, 1) = '-'
    ),
    value INTEGER NOT NULL CHECK (value >= 0),
    notes TEXT,
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0),
    UNIQUE (habit_id, date)
);

CREATE INDEX habits_archive_start_idx ON habits(archive_date, start_date);
CREATE INDEX habit_rules_habit_date_idx ON habit_rules(habit_id, effective_date);
CREATE INDEX habit_entries_habit_date_idx ON habit_entries(habit_id, date);
