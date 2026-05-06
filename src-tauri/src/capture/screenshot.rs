use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::{capture::service::CapturedClickEvent, models::AppErrorResponse};

#[derive(Debug, Clone)]
pub struct ScreenshotStorage {
    root: PathBuf,
}

impl ScreenshotStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn original_path_for_step(&self, session_id: &str, step_number: i64) -> PathBuf {
        self.root
            .join(format!("session-{}", safe_path_segment(session_id)))
            .join(format!("step-{step_number:04}-original.png"))
    }
}

#[derive(Debug, Clone)]
pub struct ScreenshotCaptureResult {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

pub fn capture_original_screenshot_for_step(
    storage: &ScreenshotStorage,
    event: &CapturedClickEvent,
    step_number: i64,
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

    let image = capture_visible_monitor(event.x, event.y)?;
    write_png(&output_path, image.width, image.height, &image.rgba)?;

    Ok(ScreenshotCaptureResult {
        path: output_path,
        width: image.width,
        height: image.height,
    })
}

struct CapturedImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
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
fn capture_visible_monitor(x: i64, y: i64) -> Result<CapturedImage, AppErrorResponse> {
    use std::{
        mem::{size_of, zeroed},
        ptr::null_mut,
    };
    use windows_sys::Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO},
        UI::WindowsAndMessaging::{GetDC, ReleaseDC},
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
            screen_dc, rect.left, rect.top, width_i32, height_i32, width, height,
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
) -> Result<CapturedImage, AppErrorResponse> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP,
        HDC, HGDIOBJ, SRCCOPY,
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
    fn screenshot_path_uses_safe_session_folder_and_step_filename() {
        let storage = ScreenshotStorage::new(PathBuf::from("screenshots"));
        let path = storage.original_path_for_step("session:one/two", 7);
        assert_eq!(
            path,
            PathBuf::from("screenshots")
                .join("session-session-one-two")
                .join("step-0007-original.png")
        );
    }
}
