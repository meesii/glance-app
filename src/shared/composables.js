import { ref } from 'vue';
import { translate_text } from './api.js';

/**
 * 翻译功能组合式函数
 * @returns {Object} 翻译相关的响应式状态和方法
 */
export function use_translate() {
    const loading = ref(false);
    const error_msg = ref('');
    const translation = ref('');
    const original = ref('');

    /**
     * 执行翻译请求（支持流式更新）
     * @param {string} text - 待翻译文本
     * @param {string} source_lang - 源语言代码
     * @param {string} target_lang - 目标语言代码
     * @param {string} [source_app] - 来源程序名称
     */
    async function do_translate(text, source_lang, target_lang, source_app = '') {
        if (!text?.trim()) return;

        translation.value = '';
        error_msg.value = '';
        loading.value = true;
        original.value = text.trim();

        let streaming = '';
        let first_chunk = true;

        try {
            const result = await translate_text({ text: text.trim(), source_lang, target_lang, source_app }, (chunk) => {
                streaming += chunk;
                translation.value = streaming;
                if (first_chunk) {
                    loading.value = false;
                    first_chunk = false;
                }
            });
            original.value = result.original || text.trim();
            translation.value = result.translation || '';
        } catch (err) {
            error_msg.value = err.message || '翻译失败';
        } finally {
            loading.value = false;
        }
    }

    return { loading, error_msg, translation, original, do_translate };
}
