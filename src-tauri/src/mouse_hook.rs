use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::OnceLock;

static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);
static ENABLED: AtomicBool = AtomicBool::new(false);
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg(target_os = "windows")]
extern "system" {
    fn SetWindowsHookExW(
        id_hook: i32,
        lpfn: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
        hmod: isize,
        thread_id: u32,
    ) -> isize;
    fn CallNextHookEx(hhk: isize, code: i32, w_param: usize, l_param: isize) -> isize;
    fn GetMessageW(msg: *mut MsgW, hwnd: isize, filter_min: u32, filter_max: u32) -> i32;
    fn GetModuleHandleW(module_name: *const u16) -> isize;
    fn UnhookWindowsHookEx(hhk: isize) -> i32;
}

/// WH_MOUSE_LL = 14
const WH_MOUSE_LL: i32 = 14;
/// WM_MBUTTONDOWN = 0x0207
const WM_MBUTTONDOWN: usize = 0x0207;
/// WM_MBUTTONUP = 0x0208
const WM_MBUTTONUP: usize = 0x0208;
/// WM_QUIT = 0x0012
#[allow(dead_code)]
const WM_QUIT: u32 = 0x0012;

#[cfg(target_os = "windows")]
#[repr(C)]
struct MsgW {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

/// 启动鼠标中键全局监听
pub fn start(app: &tauri::AppHandle) {
    APP_HANDLE.get_or_init(|| app.clone());

    if HOOK_HANDLE.load(Ordering::SeqCst) != 0 {
        ENABLED.store(true, Ordering::SeqCst);
        return;
    }

    ENABLED.store(true, Ordering::SeqCst);

    std::thread::spawn(|| {
        #[cfg(target_os = "windows")]
        unsafe {
            let hmod = GetModuleHandleW(std::ptr::null());
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0);
            if hook == 0 {
                eprintln!("鼠标钩子安装失败");
                return;
            }
            HOOK_HANDLE.store(hook, Ordering::SeqCst);

            let mut msg: MsgW = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                /* 消息泵保持钩子线程存活 */
            }

            UnhookWindowsHookEx(hook);
            HOOK_HANDLE.store(0, Ordering::SeqCst);
        }
    });
}

/// 停止鼠标中键监听
pub fn stop() {
    ENABLED.store(false, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 && ENABLED.load(Ordering::SeqCst) {
        if w_param == WM_MBUTTONDOWN {
            /* 吞掉中键按下，防止目标程序清除选区 */
            return 1;
        }
        if w_param == WM_MBUTTONUP {
            if let Some(app) = APP_HANDLE.get() {
                let app = app.clone();
                std::thread::spawn(move || {
                    crate::selection::direct_translate(&app);
                });
            }
            /* 吞掉中键松开 */
            return 1;
        }
    }
    CallNextHookEx(HOOK_HANDLE.load(Ordering::SeqCst), code, w_param, l_param)
}
