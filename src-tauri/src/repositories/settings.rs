use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::models::{AppErrorResponse, AppSettings, UpdateSettingsInput};

const SETTING_KEYS: [&str; 6] = [
    "screenshot_mode",
    "click_debounce_ms",
    "include_timestamps_in_export",
    "include_click_markers",
    "privacy_reminder_before_export",
    "default_export_directory",
];

pub fn get_settings(connection: &Connection) -> Result<AppSettings, AppErrorResponse> {
    let mut statement = connection
        .prepare("SELECT key, value FROM app_settings")
        .map_err(to_database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(to_database_error)?;

    let mut values = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(to_database_error)?;
        values.insert(key, value);
    }

    for key in SETTING_KEYS {
        if key == "screenshot_mode" {
            continue;
        }

        if !values.contains_key(key) {
            return Err(AppErrorResponse::new(
                "settings_missing_key",
                format!("The required setting '{key}' is missing."),
            ));
        }
    }

    Ok(AppSettings {
        screenshot_mode: values
            .get("screenshot_mode")
            .map(|value| normalize_screenshot_mode(value))
            .unwrap_or("clicked_monitor")
            .to_string(),
        click_debounce_ms: required_string(&values, "click_debounce_ms")?
            .parse::<i64>()
            .map_err(|_| {
                AppErrorResponse::new("settings_invalid_value", "Click debounce must be a number.")
            })?,
        include_timestamps_in_export: parse_bool(&values, "include_timestamps_in_export")?,
        include_click_markers: parse_bool(&values, "include_click_markers")?,
        privacy_reminder_before_export: parse_bool(&values, "privacy_reminder_before_export")?,
        default_export_directory: normalize_optional_path(required_string(
            &values,
            "default_export_directory",
        )?),
    })
}

pub fn update_settings(
    connection: &Connection,
    input: UpdateSettingsInput,
) -> Result<AppSettings, AppErrorResponse> {
    if let Some(screenshot_mode) = input.screenshot_mode {
        upsert_setting(
            connection,
            "screenshot_mode",
            normalize_screenshot_mode(&screenshot_mode),
        )?;
    }

    if let Some(click_debounce_ms) = input.click_debounce_ms {
        if click_debounce_ms < 0 {
            return Err(AppErrorResponse::new(
                "settings_invalid_value",
                "Click debounce cannot be negative.",
            ));
        }
        upsert_setting(
            connection,
            "click_debounce_ms",
            &click_debounce_ms.to_string(),
        )?;
    }

    if let Some(value) = input.include_timestamps_in_export {
        upsert_setting(
            connection,
            "include_timestamps_in_export",
            bool_string(value),
        )?;
    }

    if let Some(value) = input.include_click_markers {
        upsert_setting(connection, "include_click_markers", bool_string(value))?;
    }

    if let Some(value) = input.privacy_reminder_before_export {
        upsert_setting(
            connection,
            "privacy_reminder_before_export",
            bool_string(value),
        )?;
    }

    if let Some(value) = input.default_export_directory {
        upsert_setting(connection, "default_export_directory", &value)?;
    }

    get_settings(connection)
}

fn upsert_setting(connection: &Connection, key: &str, value: &str) -> Result<(), AppErrorResponse> {
    connection
        .execute(
            "INSERT INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            params![key, value],
        )
        .map_err(to_database_error)?;
    Ok(())
}

fn required_string(
    values: &HashMap<String, String>,
    key: &str,
) -> Result<String, AppErrorResponse> {
    values.get(key).cloned().ok_or_else(|| {
        AppErrorResponse::new(
            "settings_missing_key",
            format!("The required setting '{key}' is missing."),
        )
    })
}

fn parse_bool(values: &HashMap<String, String>, key: &str) -> Result<bool, AppErrorResponse> {
    match required_string(values, key)?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(AppErrorResponse::new(
            "settings_invalid_value",
            format!("The setting '{key}' must be true or false."),
        )),
    }
}

fn normalize_screenshot_mode(value: &str) -> &'static str {
    match value {
        "clicked_window" => "clicked_window",
        "clicked_monitor" => "clicked_monitor",
        _ => "clicked_monitor",
    }
}

fn normalize_optional_path(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn bool_string(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn to_database_error(error: rusqlite::Error) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "database_error",
        "The local app database could not be read.",
        error.to_string(),
    )
}
