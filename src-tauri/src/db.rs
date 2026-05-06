use rusqlite::Connection;
use std::{path::PathBuf, sync::Mutex};

pub struct AppDatabase {
    pub connection: Mutex<Connection>,
    pub path: PathBuf,
}

pub fn initialize_database(data_dir: PathBuf) -> Result<AppDatabase, String> {
    std::fs::create_dir_all(&data_dir)
        .map_err(|error| format!("Could not create app data directory: {error}"))?;

    let database_path = data_dir.join("stepforge.sqlite");
    let connection = Connection::open(&database_path)
        .map_err(|error| format!("Could not open SQLite database: {error}"))?;

    run_migrations(&connection)
        .map_err(|error| format!("Could not migrate SQLite database: {error}"))?;

    Ok(AppDatabase {
        connection: Mutex::new(connection),
        path: database_path,
    })
}

fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "foreign_keys", "ON")?;

    let current_version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        migration_001_initial_schema(connection)?;
        connection.pragma_update(None, "user_version", 1)?;
    }

    Ok(())
}

fn migration_001_initial_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS recording_sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL CHECK (status IN ('draft', 'recording', 'completed', 'exported', 'archived')),
            started_at TEXT NOT NULL,
            ended_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            default_export_directory TEXT,
            step_count INTEGER NOT NULL DEFAULT 0 CHECK (step_count >= 0),
            include_timestamps_default INTEGER NOT NULL DEFAULT 1 CHECK (include_timestamps_default IN (0, 1)),
            include_click_markers_default INTEGER NOT NULL DEFAULT 1 CHECK (include_click_markers_default IN (0, 1))
        );

        CREATE TABLE IF NOT EXISTS recording_steps (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            step_number INTEGER NOT NULL CHECK (step_number > 0),
            title TEXT NOT NULL,
            description TEXT,
            action_type TEXT NOT NULL CHECK (action_type IN ('click')),
            captured_at TEXT NOT NULL,
            click_x INTEGER,
            click_y INTEGER,
            monitor_id TEXT,
            app_window_title TEXT,
            process_name TEXT,
            original_screenshot_path TEXT NOT NULL,
            edited_screenshot_path TEXT,
            thumbnail_path TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES recording_sessions(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_recording_steps_active_step_number
            ON recording_steps(session_id, step_number)
            WHERE is_deleted = 0;

        CREATE INDEX IF NOT EXISTS idx_recording_steps_session_id
            ON recording_steps(session_id);

        CREATE TABLE IF NOT EXISTS screenshot_edits (
            id TEXT PRIMARY KEY,
            step_id TEXT NOT NULL,
            edit_type TEXT NOT NULL CHECK (edit_type IN ('crop', 'redact', 'highlight', 'arrow', 'text')),
            edit_data_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (step_id) REFERENCES recording_steps(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_screenshot_edits_step_id
            ON screenshot_edits(step_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS export_history (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            export_type TEXT NOT NULL CHECK (export_type IN ('docx', 'pdf')),
            output_path TEXT NOT NULL,
            exported_at TEXT NOT NULL,
            include_timestamps INTEGER NOT NULL CHECK (include_timestamps IN (0, 1)),
            include_click_markers INTEGER NOT NULL CHECK (include_click_markers IN (0, 1)),
            status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
            error_message TEXT,
            FOREIGN KEY (session_id) REFERENCES recording_sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_export_history_session_id
            ON export_history(session_id);

        INSERT OR IGNORE INTO app_settings (key, value, updated_at)
        VALUES
            ('screenshot_mode', 'clicked_monitor', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            ('click_debounce_ms', '500', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            ('include_timestamps_in_export', 'true', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            ('include_click_markers', 'true', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            ('privacy_reminder_before_export', 'true', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            ('default_export_directory', '', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
        "#,
    )
}
