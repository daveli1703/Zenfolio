CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY CHECK (version > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    checksum TEXT NOT NULL CHECK (length(checksum) = 64),
    applied_at TEXT NOT NULL CHECK (length(applied_at) > 0)
);

CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    currency_code TEXT NOT NULL CHECK (
        length(currency_code) = 3
        AND currency_code = upper(currency_code)
    ),
    currency_exponent INTEGER NOT NULL CHECK (currency_exponent BETWEEN 0 AND 3),
    application_timezone TEXT NOT NULL CHECK (length(trim(application_timezone)) > 0),
    date_format TEXT NOT NULL CHECK (
        date_format IN ('DD/MM/YYYY', 'MM/DD/YYYY', 'YYYY-MM-DD')
    ),
    time_format TEXT NOT NULL CHECK (time_format IN ('12h', '24h')),
    first_weekday INTEGER NOT NULL CHECK (first_weekday IN (1, 7)),
    theme TEXT NOT NULL CHECK (theme IN ('system', 'light', 'dark')),
    created_at TEXT NOT NULL CHECK (length(created_at) > 0),
    updated_at TEXT NOT NULL CHECK (length(updated_at) > 0)
);
