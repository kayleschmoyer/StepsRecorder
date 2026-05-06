use crate::{
    capture::service::{CaptureAdapter, CaptureEventSink},
    models::AppErrorResponse,
};

#[derive(Debug, Default)]
pub struct NoopCaptureAdapter;

impl NoopCaptureAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl CaptureAdapter for NoopCaptureAdapter {
    fn start(
        &mut self,
        session_id: String,
        _event_sink: CaptureEventSink,
    ) -> Result<(), AppErrorResponse> {
        println!(
            "capture.adapter event=start session_id={} adapter=noop reason=unsupported_platform",
            session_id
        );
        Ok(())
    }

    fn stop(&mut self, session_id: &str) -> Result<(), AppErrorResponse> {
        println!(
            "capture.adapter event=stop session_id={} adapter=noop reason=unsupported_platform",
            session_id
        );
        Ok(())
    }
}
