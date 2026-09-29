# Database recovery foundation

Zenfolio never treats an existing unreadable database as a missing database. If
the file is corrupt, foreign, newer than the application, has altered migration
checksums, or fails migration, startup enters recovery mode and preserves the
file at the location shown on screen.

Before upgrading an existing supported schema, Zenfolio creates a SQLite online
backup under the environment's `backups/migration` directory and independently
validates it. Migration does not start if backup creation or validation fails.

Manual backups are created through SQLite's online backup API. Zenfolio writes a
temporary sibling, validates application identity, known migration checksums,
database integrity, and foreign keys, then publishes the selected file. Backup
files are local and unencrypted.

Milestone 3 does not restore or replace a live database. Keep a problematic file
unchanged and retain any valid backups. Safe live-database replacement, staged
swaps, interruption recovery, retention controls, and polished restore UX are
planned for Milestone 10.
