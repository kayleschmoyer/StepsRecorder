use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    capture::{ActiveCaptureSession, CaptureState, SharedCaptureState},
    models::AppErrorResponse,
};

#[derive(Debug, Default)]
pub struct CaptureService {
    state: SharedCaptureState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedClickEvent {
    pub session_id: String,
    pub x: i64,
    pub y: i64,
    pub timestamp_ms: u128,
    pub monitor_id: Option<String>,
    pub active_window_title: Option<String>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClickContext {
    pub monitor_id: Option<String>,
    pub active_window_title: Option<String>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickIngestResult {
    Accepted,
    IgnoredDebounce,
    IgnoredInactive,
    IgnoredSessionMismatch,
}

impl CaptureService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&self, session_id: String, debounce_ms: i64) -> Result<(), AppErrorResponse> {
        let debounce_ms = debounce_ms.max(0) as u64;
        let mut state = self.lock_state()?;

        if let Some(active) = &state.active_session {
            if active.session_id == session_id {
                println!(
                    "capture.lifecycle event=start_already_active session_id={} debounce_ms={}",
                    active.session_id, active.debounce_ms
                );
                return Ok(());
            }

            println!(
                "capture.lifecycle event=switch_active_session previous_session_id={} next_session_id={}",
                active.session_id, session_id
            );
        }

        state.active_session = Some(ActiveCaptureSession {
            session_id: session_id.clone(),
            debounce_ms,
            last_accepted_click_timestamp_ms: None,
            accepted_clicks: Vec::new(),
        });

        println!(
            "capture.lifecycle event=start session_id={} debounce_ms={} mode=placeholder_no_global_hook",
            session_id, debounce_ms
        );
        Ok(())
    }

    pub fn stop(&self, session_id: &str) -> Result<(), AppErrorResponse> {
        let mut state = self.lock_state()?;
        match &state.active_session {
            Some(active) if active.session_id == session_id => {
                let accepted_click_count = active.accepted_clicks.len();
                state.active_session = None;
                println!(
                    "capture.lifecycle event=stop session_id={} accepted_placeholder_clicks={}",
                    session_id, accepted_click_count
                );
            }
            Some(active) => {
                println!(
                    "capture.lifecycle event=stop_ignored reason=session_mismatch active_session_id={} requested_session_id={}",
                    active.session_id, session_id
                );
            }
            None => {
                println!(
                    "capture.lifecycle event=stop_ignored reason=inactive requested_session_id={}",
                    session_id
                );
            }
        }
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_active(&self) -> Result<bool, AppErrorResponse> {
        Ok(self.lock_state()?.active_session.is_some())
    }

    /// Internal/test-only placeholder click pipeline.
    ///
    /// Step 6 intentionally does not install a Windows `WH_MOUSE_LL` hook,
    /// register Raw Input devices, read foreground window metadata, capture
    /// screenshots, or write `recording_steps`. Future Windows integration
    /// should feed native click metadata into this boundary after explicit user
    /// opt-in and should keep OS-specific API calls outside the recording
    /// repository layer.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn ingest_placeholder_click(
        &self,
        session_id: impl Into<String>,
        x: i64,
        y: i64,
        timestamp_ms: u128,
        context: ClickContext,
    ) -> Result<ClickIngestResult, AppErrorResponse> {
        let session_id = session_id.into();
        let mut state = self.lock_state()?;
        let Some(active) = state.active_session.as_mut() else {
            return Ok(ClickIngestResult::IgnoredInactive);
        };

        if active.session_id != session_id {
            return Ok(ClickIngestResult::IgnoredSessionMismatch);
        }

        if is_duplicate_click(
            active.last_accepted_click_timestamp_ms,
            timestamp_ms,
            active.debounce_ms,
        ) {
            println!(
                "capture.click event=ignored_duplicate session_id={} x={} y={} timestamp_ms={} debounce_ms={}",
                session_id, x, y, timestamp_ms, active.debounce_ms
            );
            return Ok(ClickIngestResult::IgnoredDebounce);
        }

        active.last_accepted_click_timestamp_ms = Some(timestamp_ms);
        active.accepted_clicks.push(CapturedClickEvent {
            session_id,
            x,
            y,
            timestamp_ms,
            monitor_id: context.monitor_id,
            active_window_title: context.active_window_title,
            process_name: context.process_name,
        });

        Ok(ClickIngestResult::Accepted)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn ingest_placeholder_click_now(
        &self,
        session_id: impl Into<String>,
        x: i64,
        y: i64,
        context: ClickContext,
    ) -> Result<ClickIngestResult, AppErrorResponse> {
        self.ingest_placeholder_click(session_id, x, y, current_timestamp_ms(), context)
    }

    #[cfg(test)]
    pub fn accepted_placeholder_clicks(&self) -> Result<Vec<CapturedClickEvent>, AppErrorResponse> {
        Ok(self
            .lock_state()?
            .active_session
            .as_ref()
            .map(|active| active.accepted_clicks.clone())
            .unwrap_or_default())
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, CaptureState>, AppErrorResponse> {
        self.state.inner.lock().map_err(|error| {
            AppErrorResponse::with_details(
                "capture_state_lock_error",
                "The click capture service is currently unavailable.",
                error.to_string(),
            )
        })
    }
}

fn is_duplicate_click(
    last_accepted_timestamp_ms: Option<u128>,
    timestamp_ms: u128,
    debounce_ms: u64,
) -> bool {
    last_accepted_timestamp_ms
        .map(|last| timestamp_ms.saturating_sub(last) < u128::from(debounce_ms))
        .unwrap_or(false)
}

fn current_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_ignores_clicks_inside_configured_window() {
        let service = CaptureService::new();
        service
            .start("session-a".to_string(), 250)
            .expect("start capture");

        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 10, 20, 1_000, ClickContext::default())
                .expect("first click"),
            ClickIngestResult::Accepted
        );
        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 11, 21, 1_249, ClickContext::default())
                .expect("duplicate click"),
            ClickIngestResult::IgnoredDebounce
        );

        let accepted = service
            .accepted_placeholder_clicks()
            .expect("accepted placeholder clicks");
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].x, 10);
        assert_eq!(accepted[0].y, 20);
    }

    #[test]
    fn debounce_accepts_click_at_or_after_configured_window() {
        let service = CaptureService::new();
        service
            .start("session-a".to_string(), 250)
            .expect("start capture");

        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 10, 20, 1_000, ClickContext::default())
                .expect("first click"),
            ClickIngestResult::Accepted
        );
        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 11, 21, 1_250, ClickContext::default())
                .expect("second click"),
            ClickIngestResult::Accepted
        );

        assert_eq!(
            service
                .accepted_placeholder_clicks()
                .expect("accepted placeholder clicks")
                .len(),
            2
        );
    }

    #[test]
    fn placeholder_pipeline_requires_active_matching_session() {
        let service = CaptureService::new();
        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 10, 20, 1_000, ClickContext::default())
                .expect("inactive click"),
            ClickIngestResult::IgnoredInactive
        );

        service
            .start("session-a".to_string(), 0)
            .expect("start capture");
        assert_eq!(
            service
                .ingest_placeholder_click("session-b", 10, 20, 1_000, ClickContext::default())
                .expect("mismatched click"),
            ClickIngestResult::IgnoredSessionMismatch
        );
    }
}
