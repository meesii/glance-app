import ToastEventBus from 'primevue/toasteventbus';

/**
 * 各级别 Toast 的默认配置
 * @type {Object<string, {life: number, closable: boolean}>}
 */
const DEFAULTS = {
    success: { life: 3000, closable: false },
    error: { life: 10000, closable: true },
    info: { life: 4000, closable: false },
    warn: { life: 5000, closable: true },
};

/**
 * 发送 Toast 消息
 * @param {string} severity - 消息级别
 * @param {string} text - 消息内容
 * @param {number} [life] - 显示时长（毫秒）
 * @param {string} [group] - Toast 分组
 */
function emit(severity, text, life, group) {
    const cfg = DEFAULTS[severity];
    ToastEventBus.emit('add', {
        severity,
        summary: text,
        life: life || cfg.life,
        group,
        closable: cfg.closable,
    });
}

/**
 * Toast 消息工具，提供四种级别的快捷方法
 * @type {Object}
 */
export default {
    /**
     * 成功提示
     * @param {string} text - 消息内容
     * @param {number} [life] - 显示时长
     * @param {string} [group] - 分组
     */
    success: (text, life, group) => emit('success', text, life, group),

    /**
     * 错误提示
     * @param {string} text - 消息内容
     * @param {number} [life] - 显示时长
     * @param {string} [group] - 分组
     */
    error: (text, life, group) => emit('error', text, life, group),

    /**
     * 信息提示
     * @param {string} text - 消息内容
     * @param {number} [life] - 显示时长
     * @param {string} [group] - 分组
     */
    info: (text, life, group) => emit('info', text, life, group),

    /**
     * 警告提示
     * @param {string} text - 消息内容
     * @param {number} [life] - 显示时长
     * @param {string} [group] - 分组
     */
    warn: (text, life, group) => emit('warn', text, life, group),
};
