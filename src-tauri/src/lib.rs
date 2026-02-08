mod tray;
mod shortcut;
mod selection;
mod translate;
mod doubao;
mod tts;
mod config;
mod mouse_hook;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::save_config,
            translate::translate_text,
            selection::get_selection,
            shortcut::update_shortcuts_cmd,
            update_mouse_hook_cmd,
            tts_speak_cmd,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            let config = config::load_config(app.handle());
            if let Err(e) = shortcut::register_shortcuts(app.handle(), &config) {
                eprintln!("注册快捷键失败：{}", e);
            }
            if config.middle_click_trigger {
                mouse_hook::start(app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

#[tauri::command]
fn update_mouse_hook_cmd(app: tauri::AppHandle, enabled: bool) {
    if enabled {
        mouse_hook::start(&app);
    } else {
        mouse_hook::stop();
    }
}

/// Tauri 命令：豆包 TTS 语音合成，返回 base64 编码的 AAC 音频
#[tauri::command]
async fn tts_speak_cmd(app: tauri::AppHandle, request: tts::TtsRequest) -> Result<String, String> {
    let config = config::load_config(&app);
    let cookie = config.doubao_cookie.unwrap_or_default();
    if cookie.is_empty() {
        return Err("请先在设置中配置豆包 Cookie".to_string());
    }
    tts::doubao_tts(&request, &cookie).await
}
