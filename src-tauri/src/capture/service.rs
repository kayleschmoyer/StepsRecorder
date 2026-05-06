use std::{
    sync::{mpsc, Arc, Mutex, MutexGuard},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use crate::{
    capture::{
        default_capture_adapter, ActiveCaptureSession, CaptureState, SharedCaptureState,
        SharedCaptureStateHandle,
    },
    models::AppErrorResponse,
    repositories::steps,
};

pub struct CaptureService {
    state: SharedCaptureStateHandle,
    adapter: std::sync::Mutex<Box<dyn CaptureAdapter>>,
    step_event_sender: Option<mpsc::Sender<CapturedClickEvent>>,
}

pub trait CaptureAdapter: Send {
    fn start(
        &mut self,
        session_id: String,
        event_sink: CaptureEventSink,
    ) -> Result<(), AppErrorResponse>;

    fn stop(&mut self, session_id: &str) -> Result<(), AppErrorResponse>;
}

#[derive(Debug, Clone)]
pub struct CaptureEventSink {
    state: SharedCaptureStateHandle,
    step_event_sender: Option<mpsc::Sender<CapturedClickEvent>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeClickEvent {
    pub session_id: String,
    pub x: i64,
    pub y: i64,
    pub timestamp_ms: u128,
    pub context: ClickContext,
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
    pub fn new(database: Arc<Mutex<Connection>>) -> Self {
        Self::with_adapter_and_database(default_capture_adapter(), Some(database))
    }

    #[cfg(test)]
    pub(crate) fn with_adapter(adapter: Box<dyn CaptureAdapter>) -> Self {
        Self::with_adapter_and_database(adapter, None)
    }

    fn with_adapter_and_database(
        adapter: Box<dyn CaptureAdapter>,
        database: Option<Arc<Mutex<Connection>>>,
    ) -> Self {
        let step_event_sender = database.map(spawn_step_persistence_worker);

        Self {
            state: Arc::new(SharedCaptureState::default()),
            adapter: std::sync::Mutex::new(adapter),
            step_event_sender,
        }
    }

    pub fn start(&self, session_id: String, debounce_ms: i64) -> Result<(), AppErrorResponse> {
        let debounce_ms = debounce_ms.max(0) as u64;
        let mut state = self.lock_state()?;
        let previous_session_id = state
            .active_session
            .as_ref()
            .map(|active| active.session_id.clone());

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
        drop(state);

        if let Some(previous_session_id) =
            previous_session_id.filter(|previous| previous != &session_id)
        {
            self.stop_adapter(&previous_session_id)?;
        }

        if let Err(error) = self.start_adapter(session_id.clone()) {
            let mut state = self.lock_state()?;
            if state
                .active_session
                .as_ref()
                .map(|active| active.session_id.as_str())
                == Some(session_id.as_str())
            {
                state.active_session = None;
            }
            return Err(error);
        }

        println!(
            "capture.lifecycle event=start session_id={} debounce_ms={} mode=native_click_adapter",
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
                drop(state);
                self.stop_adapter(session_id)?;
                println!(
                    "capture.lifecycle event=stop session_id={} accepted_clicks={}",
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

    /// Internal click pipeline shared by native adapters and tests.
    ///
    /// Step 8 routes Windows-native click metadata through this boundary only
    /// while a session is active. Accepted events are forwarded to a service-side
    /// persistence worker; this method does not capture screenshots or write image files.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn ingest_placeholder_click(
        &self,
        session_id: impl Into<String>,
        x: i64,
        y: i64,
        timestamp_ms: u128,
        context: ClickContext,
    ) -> Result<ClickIngestResult, AppErrorResponse> {
        self.ingest_native_click(NativeClickEvent {
            session_id: session_id.into(),
            x,
            y,
            timestamp_ms,
            context,
        })
    }

    pub(crate) fn ingest_native_click(
        &self,
        event: NativeClickEvent,
    ) -> Result<ClickIngestResult, AppErrorResponse> {
        self.event_sink().ingest_click(event)
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

    fn start_adapter(&self, session_id: String) -> Result<(), AppErrorResponse> {
        self.adapter
            .lock()
            .map_err(|error| {
                AppErrorResponse::with_details(
                    "capture_adapter_lock_error",
                    "The click capture adapter is currently unavailable.",
                    error.to_string(),
                )
            })?
            .start(session_id, self.event_sink())
    }

    fn stop_adapter(&self, session_id: &str) -> Result<(), AppErrorResponse> {
        self.adapter
            .lock()
            .map_err(|error| {
                AppErrorResponse::with_details(
                    "capture_adapter_lock_error",
                    "The click capture adapter is currently unavailable.",
                    error.to_string(),
                )
            })?
            .stop(session_id)
    }

    fn event_sink(&self) -> CaptureEventSink {
        CaptureEventSink::new(self.state.clone(), self.step_event_sender.clone())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, CaptureState>, AppErrorResponse> {
        lock_capture_state(&self.state)
    }
}

impl CaptureEventSink {
    pub(crate) fn new(
        state: SharedCaptureStateHandle,
        step_event_sender: Option<mpsc::Sender<CapturedClickEvent>>,
    ) -> Self {
        Self {
            state,
            step_event_sender,
        }
    }

    pub fn ingest_click(
        &self,
        event: NativeClickEvent,
    ) -> Result<ClickIngestResult, AppErrorResponse> {
        let result = ingest_click_into_state(&self.state, event)?;

        if let ClickIngestResult::Accepted = result {
            if let Some(accepted_event) = latest_accepted_click(&self.state)? {
                if let Some(sender) = &self.step_event_sender {
                    if let Err(error) = sender.send(accepted_event) {
                        eprintln!(
                            "capture.step event=queue_error message=accepted_click_not_persisted details={}",
                            error
                        );
                    }
                }
            }
        }

        Ok(result)
    }
}

fn ingest_click_into_state(
    state: &SharedCaptureState,
    event: NativeClickEvent,
) -> Result<ClickIngestResult, AppErrorResponse> {
    let mut state = lock_capture_state(state)?;
    let Some(active) = state.active_session.as_mut() else {
        return Ok(ClickIngestResult::IgnoredInactive);
    };

    if active.session_id != event.session_id {
        return Ok(ClickIngestResult::IgnoredSessionMismatch);
    }

    let monitor_id = sanitize_optional_metadata(event.context.monitor_id.clone(), 100);
    let active_window_title =
        sanitize_optional_metadata(event.context.active_window_title.clone(), 200);
    let process_name = sanitize_optional_metadata(event.context.process_name.clone(), 100);

    if is_duplicate_click(
        active.last_accepted_click_timestamp_ms,
        event.timestamp_ms,
        active.debounce_ms,
    ) {
        println!(
            "capture.click event=ignored_duplicate session_id={} x={} y={} timestamp_ms={} debounce_ms={} debounce_result=ignored monitor_id={} active_window_title={} process_name={}",
            event.session_id,
            event.x,
            event.y,
            event.timestamp_ms,
            active.debounce_ms,
            log_optional(&monitor_id),
            log_optional(&active_window_title),
            log_optional(&process_name)
        );
        return Ok(ClickIngestResult::IgnoredDebounce);
    }

    active.last_accepted_click_timestamp_ms = Some(event.timestamp_ms);
    active.accepted_clicks.push(CapturedClickEvent {
        session_id: event.session_id.clone(),
        x: event.x,
        y: event.y,
        timestamp_ms: event.timestamp_ms,
        monitor_id: monitor_id.clone(),
        active_window_title: active_window_title.clone(),
        process_name: process_name.clone(),
    });

    println!(
        "capture.click event=accepted session_id={} x={} y={} timestamp_ms={} debounce_ms={} debounce_result=accepted monitor_id={} active_window_title={} process_name={}",
        event.session_id,
        event.x,
        event.y,
        event.timestamp_ms,
        active.debounce_ms,
        log_optional(&monitor_id),
        log_optional(&active_window_title),
        log_optional(&process_name)
    );

    Ok(ClickIngestResult::Accepted)
}

fn lock_capture_state(
    state: &SharedCaptureState,
) -> Result<MutexGuard<'_, CaptureState>, AppErrorResponse> {
    state.inner.lock().map_err(|error| {
        AppErrorResponse::with_details(
            "capture_state_lock_error",
            "The click capture service is currently unavailable.",
            error.to_string(),
        )
    })
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

fn latest_accepted_click(
    state: &SharedCaptureState,
) -> Result<Option<CapturedClickEvent>, AppErrorResponse> {
    Ok(lock_capture_state(state)?
        .active_session
        .as_ref()
        .and_then(|active| active.accepted_clicks.last().cloned()))
}

fn spawn_step_persistence_worker(
    database: Arc<Mutex<Connection>>,
) -> mpsc::Sender<CapturedClickEvent> {
    let (sender, receiver) = mpsc::channel::<CapturedClickEvent>();

    thread::spawn(move || {
        for event in receiver {
            let connection = match database.lock() {
                Ok(connection) => connection,
                Err(error) => {
                    eprintln!(
                        "capture.step event=persist_error session_id={} message=database_lock_failed details={}",
                        event.session_id, error
                    );
                    continue;
                }
            };

            match steps::create_recorded_click_step(&connection, &event) {
                Ok(step) => println!(
                    "capture.step event=created session_id={} step_id={} step_number={} x={} y={} process_name={}",
                    step.session_id,
                    step.id,
                    step.step_number,
                    step.click_x.unwrap_or_default(),
                    step.click_y.unwrap_or_default(),
                    log_optional(&step.process_name)
                ),
                Err(error) => eprintln!(
                    "capture.step event=persist_error session_id={} code={} message={}",
                    event.session_id, error.code, error.message
                ),
            }
        }
    });

    sender
}

fn sanitize_optional_metadata(value: Option<String>, max_chars: usize) -> Option<String> {
    value
        .map(|value| truncate_metadata(value.trim(), max_chars))
        .filter(|value| !value.is_empty())
}

fn truncate_metadata(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() && max_chars > 1 {
        let keep = max_chars.saturating_sub(1);
        let mut readable: String = truncated.chars().take(keep).collect();
        readable.push('…');
        readable
    } else {
        truncated
    }
}

fn log_optional(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("<unavailable>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct TestAdapter {
        started: Vec<String>,
        stopped: Vec<String>,
    }

    impl CaptureAdapter for TestAdapter {
        fn start(
            &mut self,
            session_id: String,
            _event_sink: CaptureEventSink,
        ) -> Result<(), AppErrorResponse> {
            self.started.push(session_id);
            Ok(())
        }

        fn stop(&mut self, session_id: &str) -> Result<(), AppErrorResponse> {
            self.stopped.push(session_id.to_string());
            Ok(())
        }
    }

    fn test_service() -> CaptureService {
        CaptureService::with_adapter(Box::new(TestAdapter::default()))
    }

    #[test]
    fn debounce_ignores_clicks_inside_configured_window() {
        let service = test_service();
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
        let service = test_service();
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
    fn pipeline_requires_active_matching_session() {
        let service = test_service();
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

    #[test]
    fn stop_prevents_late_click_processing() {
        let service = test_service();
        service
            .start("session-a".to_string(), 0)
            .expect("start capture");
        service.stop("session-a").expect("stop capture");

        assert_eq!(
            service
                .ingest_placeholder_click("session-a", 10, 20, 1_000, ClickContext::default())
                .expect("late click"),
            ClickIngestResult::IgnoredInactive
        );
    }
}
