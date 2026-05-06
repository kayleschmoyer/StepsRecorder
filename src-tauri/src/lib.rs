mod capture;
mod commands;
mod db;
mod models;
mod repositories;

use std::{error::Error, io};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().setup(|app| {
        let app_data_dir = app.path().app_data_dir().map_err(|error| {
            setup_error(format!("Could not resolve app data directory: {error}"))
        })?;
        let data_dir = app_data_dir.join("data");
        let screenshots_root = app_data_dir.join("screenshots");
        let database = db::initialize_database(data_dir).map_err(setup_error)?;
        println!(
            "Steps Recorder SQLite database: {}",
            database.path.display()
        );
        println!(
            "Steps Recorder screenshots directory: {}",
            screenshots_root.display()
        );
        let screenshot_storage =
            capture::screenshot::ScreenshotStorage::new(screenshots_root.clone());
        let capture_service =
            capture::CaptureService::new(database.connection.clone(), screenshots_root);
        app.manage(database);
        app.manage(screenshot_storage);
        app.manage(capture_service);
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
        commands::get_step_screenshot_preview,
        commands::save_edited_screenshot,
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
        commands::get_step_screenshot_preview,
        commands::save_edited_screenshot,
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
