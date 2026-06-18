import { Channel, invoke } from '@tauri-apps/api/core';

/**
 * 将 Tauri invoke 错误转换为中文提示
 * @param {*} error - 原始错误
 * @returns {string} 中文错误提示
 */
function format_error(error) {
    const msg = typeof error === 'string' ? error : String(error);
    const lower = msg.toLowerCase();
    if (lower.includes('network') || lower.includes('connect') || lower.includes('dns')) {
        return '网络连接失败，请检查网络设置';
    }
    if (lower.includes('timeout') || lower.includes('timed out')) {
        return '翻译请求超时，请稍后重试';
    }
    return msg;
}

/**
 * 调用翻译服务（流式）
 * @param {Object} request - { text, source_lang, target_lang }
 * @param {Function} on_chunk - 流式数据回调
 * @returns {Promise<Object>} 翻译结果
 */
export async function translate_text(request, on_chunk) {
    try {
        const channel = new Channel();
        channel.onmessage = (chunk) => {
            if (typeof on_chunk === 'function') on_chunk(chunk);
        };
        return await invoke('translate_text', { request, channel });
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 获取配置
 * @returns {Promise<Object>}
 */
export async function get_config() {
    try {
        return await invoke('get_config');
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 保存配置
 * @param {Object} config - 配置对象
 */
export async function save_config(config) {
    try {
        return await invoke('save_config', { config });
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 更新快捷键
 * @param {Object} config - 配置对象
 */
export async function update_shortcuts(config) {
    try {
        return await invoke('update_shortcuts_cmd', { config });
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 更新鼠标中键触发状态
 * @param {boolean} enabled - 是否启用
 */
export async function update_mouse_hook(enabled) {
    try {
        return await invoke('update_mouse_hook_cmd', { enabled });
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 从 Chrome 获取 doubao.com 的 Cookie
 * @returns {Promise<string>} Cookie 字符串
 */
export async function get_chrome_cookies() {
    try {
        return await invoke('get_chrome_cookies_cmd');
    } catch (error) {
        throw new Error(format_error(error));
    }
}

/**
 * 调用豆包 TTS 语音合成
 * @param {string} text - 要朗读的文本
 * @param {Object} [options] - 可选参数
 * @param {string} [options.speaker] - 发音人
 * @param {number} [options.speech_rate] - 语速
 * @param {number} [options.pitch] - 音调
 * @returns {Promise<string>} base64 编码的 AAC 音频
 */
export async function tts_speak(text, options = {}) {
    try {
        const request = { text, ...options };
        return await invoke('tts_speak_cmd', { request });
    } catch (error) {
        throw new Error(format_error(error));
    }
}
