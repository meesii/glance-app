use serde::{Deserialize, Serialize};

/// 翻译请求参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranslateRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
    #[serde(default)]
    pub source_app: String,
}

/// 翻译结果
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranslateResult {
    pub original: String,
    pub translation: String,
}

/// 翻译错误类型
#[derive(Serialize, Deserialize, Debug)]
pub enum TranslateError {
    NetworkError(String),
    Timeout,
    ApiError { code: u16, message: String },
    ParseError(String),
    ConfigError(String),
}

/// Tauri 命令：执行翻译（仅豆包服务）
#[tauri::command]
pub async fn translate_text(
    request: TranslateRequest,
    app: tauri::AppHandle,
    channel: tauri::ipc::Channel<String>,
) -> Result<TranslateResult, String> {
    let config = crate::config::load_config(&app);
    let cookie = config.doubao_cookie.unwrap_or_default();

    println!("[翻译] text={}, {}->{}", request.text, request.source_lang, request.target_lang);

    if cookie.is_empty() {
        return Err("请先在设置中配置豆包 Cookie".to_string());
    }

    crate::doubao::doubao_translate(&request, &channel, &cookie, &config.doubao_model)
        .await
        .map_err(|e| {
            let msg = format!("{:?}", e);
            eprintln!("[翻译] 失败：{}", msg);
            msg
        })
}
