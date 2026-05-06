use std::{
    mem::{size_of, zeroed},
    ptr::null_mut,
    sync::{mpsc, Arc, Mutex, OnceLock},
    thread::{self, JoinHandle},
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITORINFOEXW},
    System::{
        ProcessStatus::GetModuleBaseNameW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextLengthW,
        GetWindowTextW, GetWindowThreadProcessId, PostThreadMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, HHOOK, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL,
        WM_LBUTTONDOWN, WM_QUIT,
    },
};

use crate::{
    capture::service::{CaptureAdapter, CaptureEventSink, ClickContext, NativeClickEvent},
    models::AppErrorResponse,
};

const MONITOR_DEFAULTTONEAREST: u32 = 0x00000002;

static ACTIVE_HOOK: OnceLock<Mutex<Option<Arc<HookRuntime>>>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct WindowsClickCaptureAdapter {
    runtime: Option<Arc<HookRuntime>>,
    hook_thread: Option<JoinHandle<()>>,
}

#[derive(Debug)]
struct HookRuntime {
    session_id: String,
    event_sink: CaptureEventSink,
    thread_id: u32,
    hook_handle: Mutex<Option<isize>>,
}

impl WindowsClickCaptureAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CaptureAdapter for WindowsClickCaptureAdapter {
    fn start(
        &mut self,
        session_id: String,
        event_sink: CaptureEventSink,
    ) -> Result<(), AppErrorResponse> {
        if self
            .runtime
            .as_ref()
            .map(|runtime| runtime.session_id.as_str())
            == Some(session_id.as_str())
        {
            println!(
                "capture.adapter event=start_already_active adapter=windows_wh_mouse_ll session_id={}",
                session_id
            );
            return Ok(());
        }

        if let Some(active_runtime) = self.runtime.take() {
            stop_runtime(&active_runtime, self.hook_thread.take())?;
        }

        let (ready_tx, ready_rx) = mpsc::channel();
        let thread_session_id = session_id.clone();
        let thread_sink = event_sink.clone();

        // Step 7 uses WH_MOUSE_LL rather than Raw Input because the app only needs
        // coarse user-click events while recording is active and does not yet need
        // per-device input, high-frequency mouse movement, or a dedicated window
        // message target. WH_MOUSE_LL gives screen coordinates directly and can be
        // installed/uninstalled with the recording lifecycle. The tradeoff is that
        // the callback runs on this hook thread's message pump and must return
        // quickly; therefore it only reads lightweight metadata and hands the click
        // into the in-memory service pipeline. It performs no database writes, no
        // screenshot capture, and no blocking work.
        let hook_thread = thread::spawn(move || {
            let thread_id = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
            let runtime = Arc::new(HookRuntime {
                session_id: thread_session_id,
                event_sink: thread_sink,
                thread_id,
                hook_handle: Mutex::new(None),
            });

            set_active_runtime(Some(runtime.clone()));
            let hook = unsafe {
                SetWindowsHookExW(
                    WH_MOUSE_LL,
                    Some(mouse_hook_proc),
                    null_mut::<()>() as HINSTANCE,
                    0,
                )
            };
            if hook.is_null() {
                set_active_runtime(None);
                let _ = ready_tx.send(Err(
                    "SetWindowsHookExW(WH_MOUSE_LL) returned a null hook handle".to_string(),
                ));
                return;
            }

            if let Ok(mut hook_handle) = runtime.hook_handle.lock() {
                *hook_handle = Some(hook as isize);
            }

            let _ = ready_tx.send(Ok(runtime.clone()));

            let mut message = unsafe { zeroed::<MSG>() };
            while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            if let Ok(mut hook_handle) = runtime.hook_handle.lock() {
                if let Some(hook) = hook_handle.take() {
                    unsafe {
                        UnhookWindowsHookEx(hook as HHOOK);
                    }
                }
            }
            set_active_runtime(None);
        });

        let runtime = ready_rx
            .recv()
            .map_err(|error| {
                AppErrorResponse::with_details(
                    "capture_adapter_start_error",
                    "Windows click capture could not start.",
                    error.to_string(),
                )
            })?
            .map_err(|details| {
                AppErrorResponse::with_details(
                    "capture_adapter_start_error",
                    "Windows click capture could not start.",
                    details,
                )
            })?;

        println!(
            "capture.adapter event=start adapter=windows_wh_mouse_ll session_id={}",
            session_id
        );
        self.runtime = Some(runtime);
        self.hook_thread = Some(hook_thread);
        Ok(())
    }

    fn stop(&mut self, session_id: &str) -> Result<(), AppErrorResponse> {
        let Some(runtime) = self.runtime.take() else {
            println!(
                "capture.adapter event=stop_ignored adapter=windows_wh_mouse_ll reason=inactive session_id={}",
                session_id
            );
            return Ok(());
        };

        if runtime.session_id != session_id {
            println!(
                "capture.adapter event=stop_ignored adapter=windows_wh_mouse_ll reason=session_mismatch active_session_id={} requested_session_id={}",
                runtime.session_id, session_id
            );
            self.runtime = Some(runtime);
            return Ok(());
        }

        stop_runtime(&runtime, self.hook_thread.take())?;
        println!(
            "capture.adapter event=stop adapter=windows_wh_mouse_ll session_id={}",
            session_id
        );
        Ok(())
    }
}

fn stop_runtime(
    runtime: &Arc<HookRuntime>,
    hook_thread: Option<JoinHandle<()>>,
) -> Result<(), AppErrorResponse> {
    unsafe {
        PostThreadMessageW(runtime.thread_id, WM_QUIT, 0, 0);
    }

    if let Some(hook_thread) = hook_thread {
        hook_thread.join().map_err(|_| {
            AppErrorResponse::new(
                "capture_adapter_stop_error",
                "Windows click capture did not stop cleanly.",
            )
        })?;
    }

    Ok(())
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam as u32 == WM_LBUTTONDOWN {
        let hook_struct = &*(lparam as *const MSLLHOOKSTRUCT);
        let runtime = active_runtime();
        if let Some(runtime) = runtime {
            let event = NativeClickEvent {
                session_id: runtime.session_id.clone(),
                x: hook_struct.pt.x as i64,
                y: hook_struct.pt.y as i64,
                timestamp_ms: current_timestamp_ms(),
                context: click_context(hook_struct.pt),
            };
            if let Err(error) = runtime.event_sink.ingest_click(event) {
                eprintln!(
                    "capture.click event=ingest_error adapter=windows_wh_mouse_ll code={} message={}",
                    error.code, error.message
                );
            }
        }
    }

    CallNextHookEx(null_mut(), code, wparam, lparam)
}

fn active_runtime() -> Option<Arc<HookRuntime>> {
    ACTIVE_HOOK
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|runtime| runtime.clone())
}

fn set_active_runtime(runtime: Option<Arc<HookRuntime>>) {
    if let Ok(mut active) = ACTIVE_HOOK.get_or_init(|| Mutex::new(None)).lock() {
        *active = runtime;
    }
}

fn click_context(point: POINT) -> ClickContext {
    let foreground_window = unsafe { GetForegroundWindow() };
    ClickContext {
        monitor_id: monitor_id(point),
        active_window_title: window_title(foreground_window),
        process_name: process_name(foreground_window),
    }
}

fn monitor_id(point: POINT) -> Option<String> {
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }

    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            rcMonitor: unsafe { zeroed() },
            rcWork: unsafe { zeroed() },
            dwFlags: 0,
        },
        szDevice: [0; 32],
    };

    let ok =
        unsafe { GetMonitorInfoW(monitor, &mut monitor_info as *mut MONITORINFOEXW as *mut _) };
    if ok == 0 {
        return Some(format!("hmonitor:{monitor:?}"));
    }

    wide_to_string(&monitor_info.szDevice).or_else(|| Some(format!("hmonitor:{monitor:?}")))
}

fn window_title(window: HWND) -> Option<String> {
    if window.is_null() {
        return None;
    }

    let length = unsafe { GetWindowTextLengthW(window) };
    if length <= 0 {
        return None;
    }

    let mut buffer = vec![0u16; length as usize + 1];
    let copied = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
    if copied <= 0 {
        return None;
    }

    String::from_utf16(&buffer[..copied as usize])
        .ok()
        .filter(|value| !value.is_empty())
}

fn process_name(window: HWND) -> Option<String> {
    if window.is_null() {
        return None;
    }

    let mut process_id = 0u32;
    unsafe {
        GetWindowThreadProcessId(window, &mut process_id);
    }
    if process_id == 0 {
        return None;
    }

    let process =
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };
    if process.is_null() {
        return None;
    }

    let mut buffer = vec![0u16; 260];
    let copied = unsafe {
        GetModuleBaseNameW(
            process,
            null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
    };
    unsafe {
        CloseHandle(process);
    }

    if copied == 0 {
        return None;
    }

    String::from_utf16(&buffer[..copied as usize])
        .ok()
        .filter(|value| !value.is_empty())
}

fn current_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn wide_to_string(buffer: &[u16]) -> Option<String> {
    let end = buffer
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buffer.len());
    String::from_utf16(&buffer[..end])
        .ok()
        .filter(|value| !value.is_empty())
}
