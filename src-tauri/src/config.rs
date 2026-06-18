use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

/// 豆包翻译模型枚举
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DoubaoModel {
    /// 火山翻译（translate_service: "3"）
    Volcano,
    /// 豆包翻译（translate_service: "1"）
    Doubao,
    /// 微软翻译（translate_service: "2"）
    Microsoft,
}

/// TTS 提供商枚举
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum TtsProvider {
    /// 豆包 TTS
    #[default]
    Doubao,
    /// 小米 MiMo TTS
    Mimo,
}

/// TTS 配置
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TtsConfig {
    /// TTS 提供商
    #[serde(default)]
    pub provider: TtsProvider,
    /// MiMo API Key
    #[serde(default)]
    pub mimo_api_key: Option<String>,
    /// MiMo Base URL
    #[serde(default = "default_mimo_base_url")]
    pub mimo_base_url: String,
    /// MiMo 语音角色
    #[serde(default = "default_mimo_voice")]
    pub mimo_voice: String,
    /// MiMo 模型 ID
    #[serde(default = "default_mimo_model")]
    pub mimo_model: String,
    /// MiMo 音色描述（voicedesign 模型使用）
    #[serde(default)]
    pub mimo_voice_design: Option<String>,
}

fn default_mimo_base_url() -> String {
    "https://api.xiaomimimo.com/v1".to_string()
}

fn default_mimo_voice() -> String {
    "冰糖".to_string()
}

fn default_mimo_model() -> String {
    "mimo-v2.5-tts".to_string()
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: TtsProvider::Doubao,
            mimo_api_key: None,
            mimo_base_url: default_mimo_base_url(),
            mimo_voice: default_mimo_voice(),
            mimo_model: default_mimo_model(),
            mimo_voice_design: None,
        }
    }
}

/// 应用配置
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppConfig {
    /// 豆包 Cookie
    pub doubao_cookie: Option<String>,
    /// 豆包模型
    pub doubao_model: DoubaoModel,
    /// 划词翻译快捷键
    pub shortcut_selection: String,
    /// 鼠标中键触发划词
    #[serde(default)]
    pub middle_click_trigger: bool,
    /// TTS 配置
    #[serde(default)]
    pub tts: TtsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            doubao_cookie: None,
            doubao_model: DoubaoModel::Doubao,
            shortcut_selection: "Alt+E".to_string(),
            middle_click_trigger: false,
            tts: TtsConfig::default(),
        }
    }
}

const CONFIG_FILE_NAME: &str = "config.json";

/// 从应用数据目录加载配置，失败时返回默认值
pub fn load_config(app_handle: &tauri::AppHandle) -> AppConfig {
    let dir = match app_handle.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return AppConfig::default(),
    };
    let path = dir.join(CONFIG_FILE_NAME);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return AppConfig::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// 保存配置到文件
pub fn save_config_to_file(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取数据目录失败：{}", e))?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("创建目录失败：{}", e))?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化失败：{}", e))?;
    fs::write(dir.join(CONFIG_FILE_NAME), json)
        .map_err(|e| format!("写入失败：{}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> AppConfig {
    load_config(&app)
}

#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    save_config_to_file(&app, &config)
}
