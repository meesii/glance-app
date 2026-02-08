use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState, Shortcut};
use crate::config::AppConfig;

/// 注册全局快捷键（仅划词翻译）
pub fn register_shortcuts(
    app: &tauri::AppHandle,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.shortcut_selection.is_empty() {
        if let Ok(shortcut) = config.shortcut_selection.parse::<Shortcut>() {
            let app_handle = app.clone();
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state != ShortcutState::Pressed { return; }
                let handle = app_handle.clone();
                std::thread::spawn(move || {
                    crate::selection::direct_translate(&handle);
                });
            })?;
        }
    }
    Ok(())
}

/// 注销并重新注册快捷键
pub fn update_shortcuts(
    app: &tauri::AppHandle,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().unregister_all()?;
    register_shortcuts(app, config)?;
    Ok(())
}

#[tauri::command]
pub fn update_shortcuts_cmd(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    update_shortcuts(&app, &config).map_err(|e| format!("更新快捷键失败：{}", e))
}
