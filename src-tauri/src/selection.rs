use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::Emitter;
use tauri::Listener;
use tauri::Manager;
use tauri::WebviewWindowBuilder;

/// 上次触发翻译的时间戳（毫秒），用于防抖
static LAST_TRIGGER_MS: AtomicU64 = AtomicU64::new(0);

/// 防抖间隔（毫秒）
const DEBOUNCE_MS: u64 = 300;

/// 划词检测事件数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectionEvent {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub source_app: String,
}

/// 获取当前选中文本
fn get_selected_text() -> Result<String, String> {
    get_selected_text::get_selected_text()
        .map_err(|e| format!("获取选中文本失败：{}", e))
}

#[tauri::command]
pub fn get_selection() -> Result<String, String> {
    get_selected_text()
}

/// 获取鼠标位置（Windows API）
fn get_mouse_position() -> (i32, i32) {
    #[cfg(target_os = "windows")]
    {
        use std::mem::MaybeUninit;
        #[repr(C)]
        struct POINT { x: i32, y: i32 }
        extern "system" {
            fn GetCursorPos(lp_point: *mut POINT) -> i32;
        }
        unsafe {
            let mut point = MaybeUninit::<POINT>::uninit();
            if GetCursorPos(point.as_mut_ptr()) != 0 {
                let point = point.assume_init();
                return (point.x, point.y);
            }
        }
    }
    (0, 0)
}

/// 获取当前前台窗口的可执行文件名
fn get_foreground_app_name() -> String {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetForegroundWindow() -> isize;
            fn GetWindowThreadProcessId(hwnd: isize, process_id: *mut u32) -> u32;
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
            fn CloseHandle(handle: isize) -> i32;
            fn K32GetModuleFileNameExW(
                process: isize,
                module: isize,
                name: *mut u16,
                size: u32,
            ) -> u32;
        }
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 { return String::new(); }

            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 { return String::new(); }

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 { return String::new(); }

            let mut buf = [0u16; 260];
            let len = K32GetModuleFileNameExW(handle, 0, buf.as_mut_ptr(), 260);
            CloseHandle(handle);

            if len == 0 { return String::new(); }

            let path = String::from_utf16_lossy(&buf[..len as usize]);
            path.rsplit('\\').next().unwrap_or("").to_string()
        }
    }
    #[cfg(not(target_os = "windows"))]
    { String::new() }
}

/// 触发翻译：获取选中文本 + 鼠标位置，直接打开翻译弹窗
pub fn direct_translate(app: &tauri::AppHandle) {
    /* 防抖：短时间内重复触发时忽略 */
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let prev = LAST_TRIGGER_MS.swap(now, Ordering::SeqCst);
    if now.saturating_sub(prev) < DEBOUNCE_MS {
        return;
    }

    let (x, y) = get_mouse_position();
    let source_app = get_foreground_app_name();
    std::thread::sleep(std::time::Duration::from_millis(50));

    match get_selected_text() {
        Ok(text) if !text.trim().is_empty() => {
            let event = SelectionEvent { text, x, y, source_app };
            let is_new = show_translate_window(app, x, y);
            if is_new {
                let app_clone = app.clone();
                let event_clone = event.clone();
                app.once("popup-ready", move |_ev| {
                    let _ = app_clone.emit("direct-translate", &event_clone);
                });
            } else {
                let _ = app.emit("direct-translate", event);
            }
        }
        _ => {}
    }
}

/// 显示翻译弹窗窗口（跳过浮标直接显示），返回是否为新创建
fn show_translate_window(app: &tauri::AppHandle, x: i32, y: i32) -> bool {
    let popup_w = 400;
    let popup_h = 350;
    let (ox, oy) = clamp_to_screen(x + 10, y + 10, popup_w, popup_h);

    if let Some(win) = app.get_webview_window("selection-popup") {
        let _ = win.set_position(tauri::PhysicalPosition::new(ox, oy));
        let _ = win.set_always_on_top(true);
        let _ = win.show();
        let _ = win.set_focus();
        false
    } else {
        let _ = WebviewWindowBuilder::new(
            app,
            "selection-popup",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("翻译")
        .inner_size(popup_w as f64, popup_h as f64)
        .position(ox as f64, oy as f64)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .build();
        true
    }
}

/// 获取鼠标所在显示器的工作区域
fn get_monitor_bounds(x: i32, y: i32) -> (i32, i32, i32, i32) {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct RECT { left: i32, top: i32, right: i32, bottom: i32 }
        #[repr(C)]
        struct MONITORINFO { cb_size: u32, rc_monitor: RECT, rc_work: RECT, dw_flags: u32 }
        #[repr(C)]
        struct POINT { x: i32, y: i32 }
        extern "system" {
            fn MonitorFromPoint(pt: POINT, dw_flags: u32) -> isize;
            fn GetMonitorInfoW(h_monitor: isize, lpmi: *mut MONITORINFO) -> i32;
        }
        const MONITOR_DEFAULTTONEAREST: u32 = 2;
        unsafe {
            let h_mon = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
            if h_mon != 0 {
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cb_size = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(h_mon, &mut info) != 0 {
                    let rc = &info.rc_work;
                    return (rc.left, rc.top, rc.right, rc.bottom);
                }
            }
        }
    }
    (0, 0, 1920, 1080)
}

/// 将窗口坐标限制在屏幕工作区域内
fn clamp_to_screen(x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let (left, top, right, bottom) = get_monitor_bounds(x, y);
    let cx = if x + w > right { right - w } else { x };
    let cy = if y + h > bottom { bottom - h } else { y };
    (cx.max(left), cy.max(top))
}
