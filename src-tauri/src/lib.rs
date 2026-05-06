mod commands;
mod db;
mod models;
mod repositories;

use std::{error::Error, io};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().setup(|app| {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| setup_error(format!("Could not resolve app data directory: {error}")))?
            .join("data");
        let database = db::initialize_database(data_dir).map_err(setup_error)?;
        println!(
            "Steps Recorder SQLite database: {}",
            database.path.display()
        );
        app.manage(database);
        Ok(())
    });

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::get_app_version,
        commands::get_settings,
        commands::update_settings,
        commands::start_recording_session,
        commands::stop_recording_session,
        commands::get_recording_status,
        commands::list_sessions,
        commands::get_session,
        commands::update_session,
        commands::update_step,
        commands::delete_step,
        commands::reorder_steps,
        commands::list_screenshot_edits,
        commands::list_export_history,
        commands::dev_seed_sample_data,
        commands::dev_clear_seeded_data
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::get_app_version,
        commands::get_settings,
        commands::update_settings,
        commands::start_recording_session,
        commands::stop_recording_session,
        commands::get_recording_status,
        commands::list_sessions,
        commands::get_session,
        commands::update_session,
        commands::update_step,
        commands::delete_step,
        commands::reorder_steps,
        commands::list_screenshot_edits,
        commands::list_export_history
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running Steps Recorder");
}

fn setup_error(message: String) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::Other, message))
}
