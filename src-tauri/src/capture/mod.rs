use std::sync::{Arc, Mutex};

pub mod service;

#[cfg(windows)]
pub mod windows;

#[cfg(not(windows))]
pub mod noop;

pub use service::{CaptureService, CapturedClickEvent};

#[derive(Debug, Default)]
pub struct SharedCaptureState {
    pub(crate) inner: Mutex<CaptureState>,
}

#[derive(Debug, Default)]
pub(crate) struct CaptureState {
    pub active_session: Option<ActiveCaptureSession>,
}

#[derive(Debug)]
pub(crate) struct ActiveCaptureSession {
    pub session_id: String,
    pub debounce_ms: u64,
    pub last_accepted_click_timestamp_ms: Option<u128>,
    pub accepted_clicks: Vec<CapturedClickEvent>,
}

#[cfg(windows)]
pub(crate) fn default_capture_adapter() -> Box<dyn service::CaptureAdapter> {
    Box::new(windows::WindowsClickCaptureAdapter::new())
}

#[cfg(not(windows))]
pub(crate) fn default_capture_adapter() -> Box<dyn service::CaptureAdapter> {
    Box::new(noop::NoopCaptureAdapter::new())
}

pub(crate) type SharedCaptureStateHandle = Arc<SharedCaptureState>;
