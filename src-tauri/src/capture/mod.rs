use std::sync::Mutex;

pub mod service;

pub use service::CaptureService;

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
