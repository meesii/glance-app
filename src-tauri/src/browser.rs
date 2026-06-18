use std::path::PathBuf;
use std::process::Command;

const BROWSER_PATHS: &[&str] = &[
    "Google\\Chrome\\Application\\chrome.exe",
    "Microsoft\\Edge\\Application\\msedge.exe",
];

fn find_browser() -> Option<PathBuf> {
    let bases = [
        std::env::var("PROGRAMFILES").ok(),
        std::env::var("PROGRAMFILES(X86)").ok(),
        std::env::var("LOCALAPPDATA").ok(),
    ];
    for base in bases.iter().flatten() {
        for rel in BROWSER_PATHS {
            let p = PathBuf::from(base).join(rel);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

/**
 * 通过 CDP 获取所有 cookie（包括 httpOnly），
 * 流程：启动 Chrome -> 等待用户登录 -> 轮询获取 cookie
 */
pub async fn get_doubao_cookies(_app: tauri::AppHandle) -> Result<String, String> {
    let chrome_path = find_browser().ok_or("未找到 Chrome/Edge")?;

    /* 启动 Chrome 调试模式 */
    let debug_port = 9222;
    let temp_dir = std::env::temp_dir().join("glance-chrome-debug");
    let _ = Command::new(&chrome_path)
        .args([
            &format!("--remote-debugging-port={}", debug_port),
            &format!("--user-data-dir={}", temp_dir.display()),
            "https://www.doubao.com",
        ])
        .spawn()
        .map_err(|e| format!("启动浏览器失败: {}", e))?;

    /* 轮询等待调试端口就绪（最多 20 秒） */
    let info_url = format!("http://127.0.0.1:{}/json/version", debug_port);
    let mut ready = false;
    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if reqwest::get(&info_url).await.is_ok() {
            ready = true;
            break;
        }
    }
    if !ready {
        return Err("Chrome 调试端口启动超时".to_string());
    }

    /* 轮询等待用户登录并获取 cookie（最多 300 秒） */
    for _ in 0..600 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        match try_get_cookies(debug_port).await {
            Ok(cookies) => {
                if !cookies.is_empty() {
                    let count = cookies.split("; ").count();
                    println!("[Browser] 获取到 {} 条 Cookie", count);
                    return Ok(cookies);
                }
            }
            Err(_) => continue,
        }
    }

    Err("获取超时，请确保已在浏览器中登录 doubao.com".to_string())
}

/**
 * 尝试通过 CDP 获取所有 cookie，返回格式化的 cookie 字符串
 */
async fn try_get_cookies(debug_port: u16) -> Result<String, String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    /* 优先获取页面级 WebSocket URL（而非浏览器级） */
    let pages_url = format!("http://127.0.0.1:{}/json", debug_port);
    let resp = reqwest::get(&pages_url)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let pages: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    /* 找到 doubao.com 页面的 WebSocket URL */
    let ws_url = pages
        .iter()
        .find(|p| {
            p.get("url")
                .and_then(|v| v.as_str())
                .map(|u| u.contains("doubao.com"))
                .unwrap_or(false)
        })
        .and_then(|p| p.get("webSocketDebuggerUrl"))
        .and_then(|v| v.as_str())
        .ok_or("未找到 doubao.com 页面")?;

    let (mut ws, _) = connect_async(ws_url)
        .await
        .map_err(|e| format!("WS 连接失败: {}", e))?;

    /* 启用 Network 域 */
    let cmd = serde_json::json!({
        "id": 1,
        "method": "Network.enable"
    });
    ws.send(Message::Text(cmd.to_string()))
        .await
        .map_err(|e| format!("发送失败: {}", e))?;

    while let Some(msg) = ws.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            _ => continue,
        };
        let resp: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if resp.get("id").and_then(|v| v.as_i64()) == Some(1) {
            if let Some(err) = resp.get("error") {
                return Err(format!("Network.enable 失败: {}", err));
            }
            break;
        }
    }

    /* 发送 Network.getAllCookies */
    let cmd = serde_json::json!({
        "id": 2,
        "method": "Network.getAllCookies"
    });
    ws.send(Message::Text(cmd.to_string()))
        .await
        .map_err(|e| format!("发送失败: {}", e))?;

    while let Some(msg) = ws.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            _ => continue,
        };
        let resp: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if resp.get("id").and_then(|v| v.as_i64()) != Some(2) {
            continue;
        }

        if let Some(err) = resp.get("error") {
            println!("[Browser] CDP 错误: {}", err);
            return Err(format!("CDP 错误: {}", err));
        }

        let cookies = resp
            .pointer("/result/cookies")
            .and_then(|v| v.as_array());

        let Some(arr) = cookies else {
            println!("[Browser] CDP 响应: {}", resp);
            return Err("无法解析 Cookie".to_string());
        };

        let cookie_str = arr
            .iter()
            .filter_map(|c| {
                let name = c.get("name")?.as_str()?;
                let value = c.get("value")?.as_str()?;
                Some(format!("{}={}", name, value))
            })
            .collect::<Vec<_>>()
            .join("; ");

        return Ok(cookie_str);
    }

    Err("CDP 无响应".to_string())
}
