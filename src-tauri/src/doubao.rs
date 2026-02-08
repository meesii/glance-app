use std::time::Duration;
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use crate::config::DoubaoModel;
use crate::translate::{TranslateError, TranslateRequest, TranslateResult};

const REQUEST_TIMEOUT_SECS: u64 = 15;
const DOUBAO_API_URL: &str = "https://www.doubao.com/samantha/chat/completion";
const DOUBAO_QUERY_PARAMS: &str = "language=zh&browser_language=zh-CN&device_platform=web&aid=586864&real_aid=586864&pkg_type=release_version&device_id=7603986199175200090&is_new_user=0&region=CN&sys_region=CN&use-olympus-account=1&samantha_web=1&version=1.36.0&version_code=20800&pc_version=1.36.0";

#[derive(Serialize)]
struct TranslateContent {
    text: String,
    target_language: String,
    scene_id: u32,
    translate_service: String,
    web_content: String,
}

#[derive(Serialize)]
struct DoubaoRequestBody {
    messages: Vec<DoubaoMessage>,
    completion_option: CompletionOption,
    reply_id: String,
    conversation_id: String,
    section_id: String,
    local_message_id: String,
    local_conversation_id: String,
}

#[derive(Serialize)]
struct DoubaoMessage {
    content: String,
    content_type: u32,
    attachments: Vec<Value>,
    references: Vec<Value>,
}

#[derive(Serialize)]
struct CompletionOption {
    is_regen: bool,
    with_suggest: bool,
    need_create_conversation: bool,
    is_replace: bool,
    is_delete: bool,
    memory_type: u32,
    launch_stage: u32,
    event_id: String,
    origin: String,
}

fn get_service_id(model: &DoubaoModel) -> &str {
    match model {
        DoubaoModel::Doubao => "1",
        DoubaoModel::Microsoft => "2",
        DoubaoModel::Volcano => "3",
    }
}

fn build_request_body(request: &TranslateRequest, model: &DoubaoModel) -> DoubaoRequestBody {
    let web_content = if request.source_app.is_empty() {
        String::new()
    } else {
        format!("当前程序名称：{}", request.source_app)
    };

    let content = TranslateContent {
        text: request.text.clone(),
        target_language: request.target_lang.clone(),
        scene_id: 3,
        translate_service: get_service_id(model).to_string(),
        web_content,
    };
    let content_json = serde_json::to_string(&content).unwrap_or_default();

    DoubaoRequestBody {
        messages: vec![DoubaoMessage {
            content: content_json,
            content_type: 2501,
            attachments: vec![],
            references: vec![],
        }],
        completion_option: CompletionOption {
            is_regen: false,
            with_suggest: false,
            need_create_conversation: false,
            is_replace: false,
            is_delete: false,
            memory_type: 2,
            launch_stage: 1,
            event_id: "0".to_string(),
            origin: "https://www.doubao.com".to_string(),
        },
        reply_id: "0".to_string(),
        conversation_id: "0".to_string(),
        section_id: "0".to_string(),
        local_message_id: uuid::Uuid::new_v4().to_string(),
        local_conversation_id: "0".to_string(),
    }
}

fn is_stream_end(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str)
        .ok()
        .and_then(|e| e.get("event_type")?.as_u64())
        .map_or(false, |t| t == 2003)
}

/// 解析 event_type=2005 的错误事件，提取错误信息
fn parse_error_event(json_str: &str) -> Option<String> {
    let event: Value = serde_json::from_str(json_str).ok()?;
    if event.get("event_type")?.as_u64()? != 2005 { return None; }
    let event_data: Value = serde_json::from_str(event.get("event_data")?.as_str()?).ok()?;
    let code = event_data.get("code")?.as_u64().unwrap_or(0);
    let message = event_data.get("message")?.as_str().unwrap_or("未知错误");
    let detail = event_data.get("error_detail")
        .and_then(|d| d.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("");
    Some(format!("豆包 API 错误 [{}]：{}（{}）", code, message, detail))
}

fn parse_sse_event(json_str: &str) -> Option<String> {
    let event: Value = serde_json::from_str(json_str).ok()?;
    if event.get("event_type")?.as_u64()? != 2001 { return None; }
    let event_data: Value = serde_json::from_str(event.get("event_data")?.as_str()?).ok()?;
    if event_data.get("is_delta")?.as_bool()? != true { return None; }
    let content: Value = serde_json::from_str(
        event_data.get("message")?.get("content")?.as_str()?
    ).ok()?;
    let text = content.get("text")?.as_str()?;
    if text.is_empty() { return None; }
    Some(text.to_string())
}

/// 执行豆包翻译（SSE 流式）
pub async fn doubao_translate(
    request: &TranslateRequest,
    channel: &tauri::ipc::Channel<String>,
    cookie: &str,
    model: &DoubaoModel,
) -> Result<TranslateResult, TranslateError> {
    let url = format!("{}?{}", DOUBAO_API_URL, DOUBAO_QUERY_PARAMS);
    let body = build_request_body(request, model);

    println!("[豆包] 发起请求：text={}, model={:?}", request.text, model);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            eprintln!("[豆包] 创建 HTTP 客户端失败：{}", e);
            TranslateError::NetworkError(e.to_string())
        })?;

    let response = client
        .post(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36")
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
        .header("agw-js-conv", "str")
        .header("Cookie", cookie)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            eprintln!("[豆包] 请求失败：{}", e);
            if e.is_timeout() { TranslateError::Timeout }
            else { TranslateError::NetworkError(e.to_string()) }
        })?;

    let status = response.status();
    println!("[豆包] 响应：{}", status);

    if !status.is_success() {
        let err_body = response.text().await.unwrap_or_default();
        eprintln!("[豆包] 错误响应体（前 500 字符）：{}", err_body.chars().take(500).collect::<String>());
        return Err(TranslateError::ApiError {
            code: status.as_u16(),
            message: format!("豆包 API 错误：{}，响应：{}", status, err_body.chars().take(200).collect::<String>()),
        });
    }

    let mut full_translation = String::new();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    let mut stream_finished = false;
    let mut line_count = 0u32;

    while let Some(chunk) = stream.next().await {
        if stream_finished { break; }
        let chunk = chunk.map_err(|e| {
            eprintln!("[豆包] 读取流数据失败：{}", e);
            TranslateError::NetworkError(e.to_string())
        })?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();
            if line.is_empty() { continue; }

            line_count += 1;

            let json_str = line.strip_prefix("data:").map(|d| d.trim()).unwrap_or(&line);
            if json_str.is_empty() || json_str == "[DONE]" {
                continue;
            }

            if line.starts_with("event:") {
                let event_name = line.strip_prefix("event:").unwrap_or("").trim();
                if event_name.contains("error") {
                    eprintln!("[豆包] SSE 错误事件：{}", event_name);
                }
                continue;
            }

            // 检测网关错误（含 code + message，无 event_type）
            if let Ok(obj) = serde_json::from_str::<Value>(json_str) {
                if obj.get("code").is_some() && obj.get("message").is_some() && obj.get("event_type").is_none() {
                    let code = obj.get("code").and_then(|v| v.as_str()).unwrap_or("未知");
                    let msg = obj.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
                    let err_msg = format!("豆包网关错误 [{}]：{}", code, msg);
                    eprintln!("[豆包] {}", err_msg);
                    return Err(TranslateError::ApiError {
                        code: 0,
                        message: err_msg,
                    });
                }
            }

            if is_stream_end(json_str) {
                stream_finished = true;
                break;
            }

            if let Some(err_msg) = parse_error_event(json_str) {
                eprintln!("[豆包] {}", err_msg);
                return Err(TranslateError::ApiError {
                    code: 0,
                    message: err_msg,
                });
            }

            if let Some(translated) = parse_sse_event(json_str) {
                full_translation.push_str(&translated);
                let _ = channel.send(translated);
            }
        }
    }

    println!("[豆包] 完成，{}行，{}字符", line_count, full_translation.len());

    if full_translation.is_empty() {
        eprintln!("[豆包] 警告：翻译结果为空！可能 Cookie 已失效或 API 格式变更");
        return Err(TranslateError::ApiError {
            code: 0,
            message: "翻译结果为空，Cookie 可能已失效，请重新配置".to_string(),
        });
    }

    Ok(TranslateResult {
        original: request.text.clone(),
        translation: full_translation,
    })
}
