use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use crate::{capture::service::CapturedClickEvent, models::AppErrorResponse};

#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::HDC;

#[derive(Debug, Clone)]
pub struct ScreenshotStorage {
    root: PathBuf,
}

impl ScreenshotStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn original_path_for_step(&self, session_id: &str, step_number: i64) -> PathBuf {
        self.step_path_for_variant(session_id, step_number, "original")
    }

    pub fn marked_path_for_step(&self, session_id: &str, step_number: i64) -> PathBuf {
        self.step_path_for_variant(session_id, step_number, "marked")
    }

    fn step_path_for_variant(&self, session_id: &str, step_number: i64, variant: &str) -> PathBuf {
        self.root
            .join(format!("session-{}", safe_path_segment(session_id)))
            .join(format!("step-{step_number:04}-{variant}.png"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotCaptureMode {
    ClickedMonitor,
    ClickedWindow,
}

impl ScreenshotCaptureMode {
    pub fn from_setting(value: &str) -> Self {
        match value {
            "clicked_window" => Self::ClickedWindow,
            "clicked_monitor" => Self::ClickedMonitor,
            _ => Self::ClickedMonitor,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClickedMonitor => "clicked_monitor",
            Self::ClickedWindow => "clicked_window",
        }
    }
}

pub struct ScreenshotCaptureResult {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub marker_x: u32,
    pub marker_y: u32,
}

pub fn capture_original_screenshot_for_step(
    storage: &ScreenshotStorage,
    event: &CapturedClickEvent,
    step_number: i64,
    mode: ScreenshotCaptureMode,
) -> Result<ScreenshotCaptureResult, AppErrorResponse> {
    let output_path = storage.original_path_for_step(&event.session_id, step_number);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppErrorResponse::with_details(
                "screenshot_directory_error",
                "The screenshot directory could not be created.",
                error.to_string(),
            )
        })?;
    }

    let image = capture_image_for_mode(event, mode)?;
    write_png(&output_path, image.width, image.height, &image.rgba)?;

    Ok(ScreenshotCaptureResult {
        path: output_path,
        width: image.width,
        height: image.height,
        marker_x: image.marker_x,
        marker_y: image.marker_y,
    })
}

pub fn generate_marked_screenshot_for_step(
    storage: &ScreenshotStorage,
    event: &CapturedClickEvent,
    step_number: i64,
    original_capture: &ScreenshotCaptureResult,
) -> Result<ScreenshotCaptureResult, AppErrorResponse> {
    let marked_path = storage.marked_path_for_step(&event.session_id, step_number);
    if let Some(parent) = marked_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppErrorResponse::with_details(
                "screenshot_directory_error",
                "The marked screenshot directory could not be created.",
                error.to_string(),
            )
        })?;
    }

    let (width, height, mut rgba) = read_png_rgba(&original_capture.path)?;
    let marker_x = original_capture.marker_x.min(width.saturating_sub(1));
    let marker_y = original_capture.marker_y.min(height.saturating_sub(1));
    draw_click_marker(&mut rgba, width, height, marker_x, marker_y);
    write_png(&marked_path, width, height, &rgba)?;

    Ok(ScreenshotCaptureResult {
        path: marked_path,
        width,
        height,
        marker_x,
        marker_y,
    })
}

fn capture_image_for_mode(
    event: &CapturedClickEvent,
    mode: ScreenshotCaptureMode,
) -> Result<CapturedImage, AppErrorResponse> {
    match mode {
        ScreenshotCaptureMode::ClickedMonitor => capture_visible_monitor(event.x, event.y),
        ScreenshotCaptureMode::ClickedWindow => match capture_clicked_window(event.x, event.y) {
            Ok(image) => Ok(image),
            Err(error) => {
                eprintln!(
                    "capture.screenshot event=fallback requested_mode=clicked_window fallback_mode=clicked_monitor reason_code={} reason_message={}",
                    error.code, error.message
                );
                capture_visible_monitor(event.x, event.y)
            }
        },
    }
}

struct CapturedImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    marker_x: u32,
    marker_y: u32,
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), AppErrorResponse> {
    let file = File::create(path).map_err(|error| {
        AppErrorResponse::with_details(
            "screenshot_write_error",
            "The screenshot file could not be written.",
            error.to_string(),
        )
    })?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().map_err(to_png_error)?;
    png_writer.write_image_data(rgba).map_err(to_png_error)
}

fn read_png_rgba(path: &Path) -> Result<(u32, u32, Vec<u8>), AppErrorResponse> {
    let file = File::open(path).map_err(|error| {
        AppErrorResponse::with_details(
            "screenshot_preview_read_error",
            "The original screenshot file could not be read for marker generation.",
            error.to_string(),
        )
    })?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().map_err(to_png_decode_error)?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(to_png_decode_error)?;
    let bytes = &buffer[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        png::ColorType::Grayscale => bytes
            .iter()
            .flat_map(|value| [*value, *value, *value, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        png::ColorType::Indexed => {
            return Err(AppErrorResponse::new(
                "screenshot_marker_png_error",
                "Indexed-color screenshots are not supported for click marker generation.",
            ));
        }
    };

    Ok((info.width, info.height, rgba))
}

fn draw_click_marker(rgba: &mut [u8], width: u32, height: u32, marker_x: u32, marker_y: u32) {
    draw_ring(
        rgba,
        width,
        height,
        marker_x,
        marker_y,
        12.0,
        1.0,
        [230, 174, 93, 70],
    );
    draw_ring(
        rgba,
        width,
        height,
        marker_x,
        marker_y,
        8.0,
        1.5,
        [255, 255, 255, 185],
    );
    draw_ring(
        rgba,
        width,
        height,
        marker_x,
        marker_y,
        8.0,
        0.75,
        [180, 110, 48, 135],
    );
    draw_filled_circle(
        rgba,
        width,
        height,
        marker_x,
        marker_y,
        2.5,
        [204, 116, 45, 215],
    );
}

fn draw_ring(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    radius: f32,
    stroke_width: f32,
    color: [u8; 4],
) {
    let outer = radius + (stroke_width / 2.0);
    let inner = (radius - (stroke_width / 2.0)).max(0.0);
    let min_x = (center_x as f32 - outer - 1.0).floor().max(0.0) as u32;
    let min_y = (center_y as f32 - outer - 1.0).floor().max(0.0) as u32;
    let max_x = (center_x as f32 + outer + 1.0)
        .ceil()
        .min(width.saturating_sub(1) as f32) as u32;
    let max_y = (center_y as f32 + outer + 1.0)
        .ceil()
        .min(height.saturating_sub(1) as f32) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center_x as f32;
            let dy = y as f32 + 0.5 - center_y as f32;
            let distance = ((dx * dx) + (dy * dy)).sqrt();
            let outer_coverage = (outer + 0.5 - distance).clamp(0.0, 1.0);
            let inner_coverage = (distance - inner + 0.5).clamp(0.0, 1.0);
            let coverage = outer_coverage.min(inner_coverage);
            if coverage > 0.0 {
                blend_pixel_with_coverage(rgba, width, x, y, color, coverage);
            }
        }
    }
}

fn draw_filled_circle(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    radius: f32,
    color: [u8; 4],
) {
    let min_x = (center_x as f32 - radius - 1.0).floor().max(0.0) as u32;
    let min_y = (center_y as f32 - radius - 1.0).floor().max(0.0) as u32;
    let max_x = (center_x as f32 + radius + 1.0)
        .ceil()
        .min(width.saturating_sub(1) as f32) as u32;
    let max_y = (center_y as f32 + radius + 1.0)
        .ceil()
        .min(height.saturating_sub(1) as f32) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center_x as f32;
            let dy = y as f32 + 0.5 - center_y as f32;
            let distance = ((dx * dx) + (dy * dy)).sqrt();
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            if coverage > 0.0 {
                blend_pixel_with_coverage(rgba, width, x, y, color, coverage);
            }
        }
    }
}

fn blend_pixel_with_coverage(
    rgba: &mut [u8],
    width: u32,
    x: u32,
    y: u32,
    mut color: [u8; 4],
    coverage: f32,
) {
    color[3] = (f32::from(color[3]) * coverage).round().clamp(0.0, 255.0) as u8;
    blend_pixel(rgba, width, x, y, color);
}

fn blend_pixel(rgba: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let index = ((y * width + x) * 4) as usize;
    if index + 3 >= rgba.len() {
        return;
    }

    let alpha = f32::from(color[3]) / 255.0;
    let inverse_alpha = 1.0 - alpha;
    rgba[index] =
        ((f32::from(color[0]) * alpha) + (f32::from(rgba[index]) * inverse_alpha)).round() as u8;
    rgba[index + 1] = ((f32::from(color[1]) * alpha) + (f32::from(rgba[index + 1]) * inverse_alpha))
        .round() as u8;
    rgba[index + 2] = ((f32::from(color[2]) * alpha) + (f32::from(rgba[index + 2]) * inverse_alpha))
        .round() as u8;
    rgba[index + 3] = 255;
}

fn to_png_decode_error(error: png::DecodingError) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "screenshot_marker_png_error",
        "The screenshot PNG could not be decoded for marker generation.",
        error.to_string(),
    )
}

fn to_png_error(error: png::EncodingError) -> AppErrorResponse {
    AppErrorResponse::with_details(
        "screenshot_png_error",
        "The screenshot PNG could not be encoded.",
        error.to_string(),
    )
}

fn safe_path_segment(value: &str) -> String {
    let safe: String = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect();

    if safe.is_empty() {
        "unknown-session".to_string()
    } else {
        safe
    }
}

#[cfg(windows)]
fn capture_clicked_window(x: i64, y: i64) -> Result<CapturedImage, AppErrorResponse> {
    use std::ptr::null_mut;
    use windows_sys::Win32::{
        Foundation::POINT,
        Graphics::Gdi::{GetDC, ReleaseDC},
        UI::{
            HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{GetAncestor, WindowFromPoint, GA_ROOT},
        },
    };

    let previous_dpi_context =
        unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };

    let result = (|| {
        let point = POINT {
            x: clamp_i64_to_i32(x),
            y: clamp_i64_to_i32(y),
        };
        let clicked = unsafe { WindowFromPoint(point) };
        if clicked.is_null() {
            return Err(AppErrorResponse::new(
                "screenshot_window_error",
                "The clicked window could not be resolved from the click point.",
            ));
        }

        let root = unsafe { GetAncestor(clicked, GA_ROOT) };
        let window = if root.is_null() { clicked } else { root };
        validate_capture_window(window)?;

        let rect = visible_window_rect(window)?;
        if point.x < rect.left
            || point.x >= rect.right
            || point.y < rect.top
            || point.y >= rect.bottom
        {
            return Err(AppErrorResponse::new(
                "screenshot_window_error",
                "The click point was outside the resolved top-level window bounds.",
            ));
        }

        let width_i32 = rect.right.saturating_sub(rect.left);
        let height_i32 = rect.bottom.saturating_sub(rect.top);
        if width_i32 <= 0 || height_i32 <= 0 {
            return Err(AppErrorResponse::new(
                "screenshot_window_error",
                "The resolved window bounds were empty.",
            ));
        }

        let width = width_i32 as u32;
        let height = height_i32 as u32;
        let screen_dc = unsafe { GetDC(null_mut()) };
        if screen_dc.is_null() {
            return Err(AppErrorResponse::new(
                "screenshot_capture_error",
                "The screen device context could not be opened for clicked-window capture.",
            ));
        }

        let capture = unsafe {
            capture_dc_region(
                screen_dc,
                rect.left,
                rect.top,
                width_i32,
                height_i32,
                width,
                height,
                x.saturating_sub(i64::from(rect.left))
                    .clamp(0, i64::from(width.saturating_sub(1))) as u32,
                y.saturating_sub(i64::from(rect.top))
                    .clamp(0, i64::from(height.saturating_sub(1))) as u32,
            )
        };
        unsafe {
            ReleaseDC(null_mut(), screen_dc);
        }
        capture
    })();

    if !previous_dpi_context.is_null() {
        unsafe {
            SetThreadDpiAwarenessContext(previous_dpi_context);
        }
    }

    result
}

#[cfg(windows)]
fn validate_capture_window(
    window: windows_sys::Win32::Foundation::HWND,
) -> Result<(), AppErrorResponse> {
    if window.is_null() {
        return Err(AppErrorResponse::new(
            "screenshot_window_error",
            "The resolved top-level window was unavailable.",
        ));
    }

    if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(window) } == 0 {
        return Err(AppErrorResponse::new(
            "screenshot_window_error",
            "The clicked window is not visible and will not be captured.",
        ));
    }

    if unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsIconic(window) } != 0 {
        return Err(AppErrorResponse::new(
            "screenshot_window_error",
            "The clicked window is minimized and will not be captured.",
        ));
    }

    Ok(())
}

#[cfg(windows)]
fn visible_window_rect(
    window: windows_sys::Win32::Foundation::HWND,
) -> Result<windows_sys::Win32::Foundation::RECT, AppErrorResponse> {
    use std::{mem::size_of, mem::zeroed};
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
        UI::WindowsAndMessaging::GetWindowRect,
    };

    let mut rect = unsafe { zeroed::<RECT>() };
    let dwm_ok = unsafe {
        DwmGetWindowAttribute(
            window,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            &mut rect as *mut RECT as *mut _,
            size_of::<RECT>() as u32,
        )
    };

    if dwm_ok != 0 || rect.right <= rect.left || rect.bottom <= rect.top {
        let ok = unsafe { GetWindowRect(window, &mut rect) };
        if ok == 0 {
            return Err(AppErrorResponse::new(
                "screenshot_window_error",
                "The clicked window bounds could not be read.",
            ));
        }
    }

    Ok(rect)
}

#[cfg(windows)]
fn capture_visible_monitor(x: i64, y: i64) -> Result<CapturedImage, AppErrorResponse> {
    use std::{
        mem::{size_of, zeroed},
        ptr::null_mut,
    };
    use windows_sys::Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{GetDC, GetMonitorInfoW, MonitorFromPoint, ReleaseDC, MONITORINFO},
    };

    const MONITOR_DEFAULTTONEAREST: u32 = 0x00000002;

    let point = POINT {
        x: clamp_i64_to_i32(x),
        y: clamp_i64_to_i32(y),
    };
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return Err(AppErrorResponse::new(
            "screenshot_monitor_error",
            "The clicked monitor could not be resolved.",
        ));
    }

    let mut monitor_info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        rcMonitor: unsafe { zeroed::<RECT>() },
        rcWork: unsafe { zeroed::<RECT>() },
        dwFlags: 0,
    };
    let ok = unsafe { GetMonitorInfoW(monitor, &mut monitor_info) };
    if ok == 0 {
        return Err(AppErrorResponse::new(
            "screenshot_monitor_error",
            "The clicked monitor bounds could not be read.",
        ));
    }

    let rect = monitor_info.rcMonitor;
    let width_i32 = rect.right.saturating_sub(rect.left);
    let height_i32 = rect.bottom.saturating_sub(rect.top);
    if width_i32 <= 0 || height_i32 <= 0 {
        return Err(AppErrorResponse::new(
            "screenshot_monitor_error",
            "The clicked monitor bounds were empty.",
        ));
    }

    let width = width_i32 as u32;
    let height = height_i32 as u32;
    let screen_dc = unsafe { GetDC(null_mut()) };
    if screen_dc.is_null() {
        return Err(AppErrorResponse::new(
            "screenshot_capture_error",
            "The screen device context could not be opened.",
        ));
    }

    let result = unsafe {
        capture_dc_region(
            screen_dc,
            rect.left,
            rect.top,
            width_i32,
            height_i32,
            width,
            height,
            x.saturating_sub(i64::from(rect.left))
                .clamp(0, i64::from(width.saturating_sub(1))) as u32,
            y.saturating_sub(i64::from(rect.top))
                .clamp(0, i64::from(height.saturating_sub(1))) as u32,
        )
    };
    unsafe {
        ReleaseDC(null_mut(), screen_dc);
    }
    return result;
}

#[cfg(windows)]
unsafe fn capture_dc_region(
    screen_dc: HDC,
    left: i32,
    top: i32,
    width_i32: i32,
    height_i32: i32,
    width: u32,
    height: u32,
    marker_x: u32,
    marker_y: u32,
) -> Result<CapturedImage, AppErrorResponse> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP,
        HGDIOBJ, SRCCOPY,
    };

    let memory_dc = CreateCompatibleDC(screen_dc);
    if memory_dc.is_null() {
        return Err(AppErrorResponse::new(
            "screenshot_capture_error",
            "A compatible screenshot device context could not be created.",
        ));
    }

    let bitmap = CreateCompatibleBitmap(screen_dc, width_i32, height_i32);
    if bitmap.is_null() {
        DeleteDC(memory_dc);
        return Err(AppErrorResponse::new(
            "screenshot_capture_error",
            "A screenshot bitmap could not be created.",
        ));
    }

    let previous = SelectObject(memory_dc, bitmap as HGDIOBJ);
    let blit_ok = BitBlt(
        memory_dc,
        0,
        0,
        width_i32,
        height_i32,
        screen_dc,
        left,
        top,
        SRCCOPY | CAPTUREBLT,
    );
    let mut pixels = vec![0u8; width as usize * height as usize * 4];

    let result = if blit_ok == 0 {
        Err(AppErrorResponse::new(
            "screenshot_capture_error",
            "The visible monitor could not be copied into a screenshot bitmap.",
        ))
    } else {
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width_i32,
                biHeight: -height_i32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [zeroed()],
        };
        let rows = GetDIBits(
            memory_dc,
            bitmap as HBITMAP,
            0,
            height,
            pixels.as_mut_ptr() as *mut _,
            &mut bitmap_info,
            DIB_RGB_COLORS,
        );
        if rows == 0 {
            Err(AppErrorResponse::new(
                "screenshot_capture_error",
                "Screenshot pixels could not be read from the bitmap.",
            ))
        } else {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
                pixel[3] = 255;
            }
            Ok(CapturedImage {
                width,
                height,
                rgba: pixels,
                marker_x,
                marker_y,
            })
        }
    };

    if !previous.is_null() {
        SelectObject(memory_dc, previous);
    }
    DeleteObject(bitmap as HGDIOBJ);
    DeleteDC(memory_dc);
    result
}

#[cfg(windows)]
fn clamp_i64_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(not(windows))]
fn capture_clicked_window(_x: i64, _y: i64) -> Result<CapturedImage, AppErrorResponse> {
    Err(AppErrorResponse::new(
        "screenshot_unsupported_platform",
        "Clicked-window screenshot capture is only implemented for Windows in Step 11.",
    ))
}

#[cfg(not(windows))]
fn capture_visible_monitor(_x: i64, _y: i64) -> Result<CapturedImage, AppErrorResponse> {
    Err(AppErrorResponse::new(
        "screenshot_unsupported_platform",
        "Screenshot capture is only implemented for Windows in Step 9.",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_capture_mode_defaults_unknown_values_to_clicked_monitor() {
        assert_eq!(
            ScreenshotCaptureMode::from_setting("clicked_monitor"),
            ScreenshotCaptureMode::ClickedMonitor
        );
        assert_eq!(
            ScreenshotCaptureMode::from_setting("clicked_window"),
            ScreenshotCaptureMode::ClickedWindow
        );
        assert_eq!(
            ScreenshotCaptureMode::from_setting("unexpected"),
            ScreenshotCaptureMode::ClickedMonitor
        );
        assert_eq!(
            ScreenshotCaptureMode::from_setting(""),
            ScreenshotCaptureMode::ClickedMonitor
        );
    }

    #[test]
    fn screenshot_path_uses_safe_session_folder_and_step_filename() {
        let storage = ScreenshotStorage::new(PathBuf::from("screenshots"));
        let original_path = storage.original_path_for_step("session:one/two", 7);
        let marked_path = storage.marked_path_for_step("session:one/two", 7);

        assert_eq!(
            original_path,
            PathBuf::from("screenshots")
                .join("session-session-one-two")
                .join("step-0007-original.png")
        );
        assert_eq!(
            marked_path,
            PathBuf::from("screenshots")
                .join("session-session-one-two")
                .join("step-0007-marked.png")
        );
    }

    #[test]
    fn marked_screenshot_is_derived_without_overwriting_original() {
        let root =
            std::env::temp_dir().join(format!("steps-recorder-marker-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let storage = ScreenshotStorage::new(root.clone());
        let event = CapturedClickEvent {
            session_id: "session-marker".to_string(),
            x: 10,
            y: 10,
            timestamp_ms: 0,
            monitor_id: None,
            active_window_title: None,
            process_name: None,
        };
        let original_path = storage.original_path_for_step(&event.session_id, 1);
        std::fs::create_dir_all(original_path.parent().expect("original parent"))
            .expect("create screenshot test directory");
        let pixels = vec![18u8; 80 * 80 * 4];
        write_png(&original_path, 80, 80, &pixels).expect("write original png");
        let original_bytes = std::fs::read(&original_path).expect("read original before marker");
        let original_capture = ScreenshotCaptureResult {
            path: original_path.clone(),
            width: 80,
            height: 80,
            marker_x: 40,
            marker_y: 40,
        };

        let marked_capture =
            generate_marked_screenshot_for_step(&storage, &event, 1, &original_capture)
                .expect("generate marked screenshot");

        assert_eq!(
            marked_capture.path,
            storage.marked_path_for_step(&event.session_id, 1)
        );
        assert!(marked_capture.path.exists());
        assert_eq!(
            std::fs::read(&original_path).expect("read original after marker"),
            original_bytes
        );
        assert_ne!(
            std::fs::read(&marked_capture.path).expect("read marked screenshot"),
            original_bytes
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
