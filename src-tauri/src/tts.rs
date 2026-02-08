use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use rand::Rng;
use tokio_tungstenite::{connect_async, tungstenite};

const TTS_WS_BASE: &str = "wss://ws-samantha.doubao.com/samantha/audio/tts";
const DEFAULT_SPEAKER: &str = "zh_female_taozi_conversation_v4_wvae_bigtts";
const DEFAULT_FORMAT: &str = "aac";

/// TTS 请求参数
#[derive(serde::Deserialize, Clone, Debug)]
pub struct TtsRequest {
    pub text: String,
    #[serde(default = "default_speaker")]
    pub speaker: String,
    #[serde(default)]
    pub speech_rate: i32,
    #[serde(default)]
    pub pitch: i32,
}

fn default_speaker() -> String {
    DEFAULT_SPEAKER.to_string()
}

/// 生成随机 device_id（19 位数字字符串）
fn random_device_id() -> String {
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen_range(1_000_000_000_000_000_000..9_999_999_999_999_999_999);
    id.to_string()
}

/// 生成随机 WebSocket Key（16 字节 base64）
fn random_ws_key() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill(&mut buf);
    BASE64.encode(buf)
}

/// 构建 WebSocket 连接 URL
fn build_ws_url(request: &TtsRequest) -> String {
    format!(
        "{}?speaker={}&format={}&mode=0&speech_rate={}&pitch={}&language=zh\
         &browser_language=zh-CN&device_platform=web&aid=586864&real_aid=586864\
         &pkg_type=release_version&device_id={}&is_new_user=0&region=CN&sys_region=CN\
         &use-olympus-account=1&samantha_web=1&version=1.36.0&version_code=20800\
         &pc_version=1.36.0",
        TTS_WS_BASE,
        request.speaker,
        DEFAULT_FORMAT,
        request.speech_rate,
        request.pitch,
        random_device_id(),
    )
}

/// 执行豆包 TTS，返回完整 AAC 音频数据（base64 编码）
pub async fn doubao_tts(request: &TtsRequest, cookie: &str) -> Result<String, String> {
    let url = build_ws_url(request);
    let uri: Uri = url.parse().map_err(|e| format!("URL 解析失败：{}", e))?;

    let host = uri.host().unwrap_or("ws-samantha.doubao.com");
    let ws_key = random_ws_key();

    let req = tungstenite::http::Request::builder()
        .uri(&url)
        .header("Host", host)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", &ws_key)
        .header("Cookie", cookie)
        .body(())
        .map_err(|e| format!("构建请求失败：{}", e))?;

    println!("[TTS] 连接 WebSocket：speaker={}", request.speaker);

    let (ws_stream, _) = connect_async(req)
        .await
        .map_err(|e| format!("WebSocket 连接失败：{}", e))?;

    let (mut writer, mut reader) = ws_stream.split();

    // 等待 open_success 或第一条消息
    let mut got_open = false;
    while let Some(msg) = reader.next().await {
        let msg = msg.map_err(|e| format!("读取消息失败：{}", e))?;
        if let tungstenite::Message::Text(txt) = &msg {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(txt) {
                if json.get("event").and_then(|v| v.as_str()) == Some("open_success") {
                    got_open = true;
                    break;
                }
            }
        }
    }

    if !got_open {
        return Err("未收到 open_success 事件".to_string());
    }

    // 发送文本
    let text_msg = serde_json::json!({
        "event": "text",
        "podcast_extra": { "role": "" },
        "text": request.text,
    });
    writer
        .send(tungstenite::Message::Text(text_msg.to_string().into()))
        .await
        .map_err(|e| format!("发送文本失败：{}", e))?;

    // 发送 finish 信号
    let finish_msg = serde_json::json!({ "event": "finish" });
    writer
        .send(tungstenite::Message::Text(finish_msg.to_string().into()))
        .await
        .map_err(|e| format!("发送 finish 失败：{}", e))?;

    println!("[TTS] 已发送文本（{}字符），等待音频数据…", request.text.len());

    // 收集音频数据
    let mut audio_buf: Vec<u8> = Vec::new();
    let mut finished = false;

    while let Some(msg) = reader.next().await {
        if finished { break; }
        let msg = msg.map_err(|e| format!("读取音频数据失败：{}", e))?;
        match msg {
            tungstenite::Message::Binary(data) => {
                audio_buf.extend_from_slice(&data);
            }
            tungstenite::Message::Text(txt) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                    let event = json.get("event").and_then(|v| v.as_str()).unwrap_or("");
                    let code = json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                    match event {
                        "finish" => {
                            finished = true;
                        }
                        "sentence_start" | "sentence_end" => {
                            // 正常流程事件，忽略
                        }
                        _ if code != 0 => {
                            let message = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
                            return Err(format!("TTS 错误 [{}]：{}", code, message));
                        }
                        _ => {}
                    }
                }
            }
            tungstenite::Message::Close(_) => break,
            _ => {}
        }
    }

    if audio_buf.is_empty() {
        return Err("未收到音频数据".to_string());
    }

    println!("[TTS] 完成，音频大小：{} 字节", audio_buf.len());

    Ok(BASE64.encode(&audio_buf))
}
