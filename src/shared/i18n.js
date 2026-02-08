/**
 * 支持的语言列表
 * @type {Array<{code: string, label: string}>}
 */
export const LANGUAGES = [
    { code: 'auto', label: '自动检测' },
    { code: 'zh', label: '中文简体' },
    { code: 'zh-TW', label: '中文繁体' },
    { code: 'en', label: '英语' },
    { code: 'ja', label: '日语' },
    { code: 'ko', label: '韩语' },
    { code: 'fr', label: '法语' },
    { code: 'de', label: '德语' },
    { code: 'es', label: '西班牙语' },
    { code: 'ru', label: '俄语' },
    { code: 'pt', label: '葡萄牙语' },
    { code: 'it', label: '意大利语' },
    { code: 'ar', label: '阿拉伯语' },
    { code: 'th', label: '泰语' },
    { code: 'vi', label: '越南语' },
];

/**
 * 获取目标语言列表（排除"自动检测"）
 * @returns {Array<{code: string, label: string}>}
 */
export function target_langs() {
    return LANGUAGES.filter((item) => item.code !== 'auto');
}
