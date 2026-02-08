use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
    Manager,
};
use tauri::WebviewWindowBuilder;

/// 初始化系统托盘
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().expect("未找到图标"))
        .tooltip("Glance 划词翻译")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app_handle, event| match event.id.as_ref() {
            "settings" => show_settings_window(app_handle),
            "quit" => app_handle.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                show_settings_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// 打开或显示设置窗口
fn show_settings_window(app_handle: &tauri::AppHandle) {
    if let Some(win) = app_handle.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(
            app_handle,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("Glance - 设置")
        .inner_size(520.0, 480.0)
        .center()
        .resizable(false)
        .maximizable(false)
        .decorations(true)
        .build();
    }
}
