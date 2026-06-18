/// MiMo TTS 请求参数
#[derive(serde::Deserialize, Clone, Debug)]
pub struct MimoTtsRequest {
    pub text: String,
    #[serde(default = "default_voice")]
    pub voice: String,
    #[serde(default = "default_model")]
    pub model: String,
    /// voicedesign 模型的音色描述
    #[serde(default)]
    pub voice_design: Option<String>,
}

fn default_voice() -> String {
    "冰糖".to_string()
}

fn default_model() -> String {
    "mimo-v2.5-tts".to_string()
}

/// 调用小米 MiMo TTS，返回 base64 编码的音频数据
pub async fn mimo_tts(request: &MimoTtsRequest, api_key: &str, base_url: &str) -> Result<String, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let is_voicedesign = request.model == "mimo-v2.5-tts-voicedesign";

    let audio = if is_voicedesign {
        serde_json::json!({ "format": "wav" })
    } else {
        serde_json::json!({ "format": "wav", "voice": request.voice })
    };

    /* voicedesign：user=音色描述（必填），assistant=待合成文本（原样朗读）
       普通模型：assistant=待合成文本 */
    let messages = if is_voicedesign {
        let desc = request.voice_design.as_deref().filter(|s| !s.is_empty()).unwrap_or("标准播音腔，声音清晰自然");
        serde_json::json!([
            { "role": "user", "content": desc },
            { "role": "assistant", "content": request.text }
        ])
    } else {
        serde_json::json!([
            { "role": "assistant", "content": request.text }
        ])
    };

    let body = serde_json::json!({
        "model": request.model,
        "messages": messages,
        "audio": audio,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("MiMo TTS 请求失败：{}", e))?;

    let status = resp.status();
    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("读取 MiMo 响应失败：{}", e))?;

    if !status.is_success() {
        return Err(format!("MiMo TTS 错误 [{}]：{}", status, resp_text));
    }

    let json: serde_json::Value = serde_json::from_str(&resp_text)
        .map_err(|e| format!("解析 MiMo 响应失败：{}，原始：{}", e, &resp_text[..200.min(resp_text.len())]))?;

    let audio_data = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("audio"))
        .and_then(|a| a.get("data"))
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            let snippet = &resp_text[..500.min(resp_text.len())];
            format!("MiMo 响应中未找到音频数据：{}", snippet)
        })?;

    Ok(audio_data.to_string())
}
