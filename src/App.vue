<script setup>
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { marked } from 'marked';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import LangSelector from './components/LangSelector.vue';
import { tts_speak } from './shared/api.js';
import { use_translate } from './shared/composables.js';

const { loading, error_msg, translation, original, do_translate } = use_translate();

/**
 * 将 translation 的 markdown 内容解析为 HTML
 * @type {import('vue').ComputedRef<string>}
 */
const rendered = computed(() => {
    if (!translation.value) return '';
    return marked.parse(translation.value);
});

const src_lang = ref('auto');
const tgt_lang = ref('zh');
const pinned = ref(false);
const text = ref('');
const src_app = ref('');

/**
 * localStorage 键名，用于持久化 pin 状态
 * @type {string}
 */
const PIN_KEY = 'glance_popup_pinned';

/**
 * localStorage 键名，用于持久化自动朗读状态
 * @type {string}
 */
const AUTO_SPEAK_KEY = 'glance_auto_speak';

/**
 * 是否开启自动朗读原文
 * @type {import('vue').Ref<boolean>}
 */
const auto_speak = ref(localStorage.getItem(AUTO_SPEAK_KEY) === 'true');

/**
 * 翻译操作集合
 * @type {Object}
 */
const trans = {
    /**
     * 执行翻译
     * @param {string} val - 待翻译文本
     * @param {string} [app] - 来源程序名称
     */
    async run(val, app) {
        if (!val?.trim()) return;
        text.value = val.trim();
        if (app !== undefined) src_app.value = app;
        await do_translate(val, src_lang.value, tgt_lang.value, src_app.value);
    },

    /**
     * 刷新当前翻译
     * @returns {Promise<void>}
     */
    async refresh() {
        if (text.value) await trans.run(text.value);
    },
};

/**
 * 语音朗读操作集合
 * @type {Object}
 */
const speech = {
    /** @private 当前播放的 Audio 实例 */
    audio: null,

    /** @private 音频缓存，key 为文本内容，value 为 base64 音频数据 */
    cache: new Map(),

    /** @private 当前朗读来源标记（original / result） */
    source: ref(''),

    /** 是否正在请求 TTS */
    fetching: ref(false),

    /** 是否正在播放音频 */
    playing: ref(false),

    /**
     * 判断指定来源是否处于活跃状态（请求中或播放中）
     * @param {string} src - 来源标记
     * @returns {boolean}
     */
    is_active(src) {
        return speech.source.value === src && (speech.fetching.value || speech.playing.value);
    },

    /**
     * 通过豆包 TTS 朗读指定文本，再次调用同来源则停止
     * @param {string} val - 文本内容
     * @param {string} src - 来源标记（original / result）
     */
    async speak(val, src) {
        if (!val?.trim()) return;
        if (speech.is_active(src)) {
            speech.stop();
            return;
        }
        speech.stop();
        speech.source.value = src;
        try {
            let base64 = speech.cache.get(val);
            if (!base64) {
                speech.fetching.value = true;
                base64 = await tts_speak(val);
                if (speech.source.value !== src) return;
                speech.cache.set(val, base64);
                speech.fetching.value = false;
            }
            const audio = new Audio(`data:audio/aac;base64,${base64}`);
            speech.audio = audio;
            speech.playing.value = true;
            audio.onended = () => {
                speech.playing.value = false;
                speech.source.value = '';
            };
            audio.onerror = () => {
                speech.playing.value = false;
                speech.source.value = '';
            };
            await audio.play();
        } catch (err) {
            console.error('TTS 朗读失败：', err);
            speech.fetching.value = false;
            speech.playing.value = false;
            speech.source.value = '';
        }
    },

    /**
     * 停止当前朗读
     */
    stop() {
        if (speech.audio) {
            speech.audio.pause();
            speech.audio = null;
        }
        speech.fetching.value = false;
        speech.playing.value = false;
        speech.source.value = '';
    },

    /**
     * 朗读原文
     */
    original() {
        speech.speak(original.value || text.value, 'original');
    },

    /**
     * 朗读译文
     */
    result() {
        speech.speak(translation.value, 'result');
    },
};

/**
 * 剪贴板操作集合
 * @type {Object}
 */
const clip = {
    /**
     * 复制文本到剪贴板
     * @param {string} val - 文本内容
     */
    async copy(val) {
        if (!val) return;
        try {
            await writeText(val);
        } catch (err) {
            console.error('复制失败：', err);
        }
    },

    /**
     * 复制原文到剪贴板
     * @returns {Promise<void>}
     */
    async original() {
        await clip.copy(original.value || text.value);
    },

    /**
     * 复制译文到剪贴板
     * @returns {Promise<void>}
     */
    async result() {
        await clip.copy(translation.value);
    },
};

/**
 * 窗口操作集合
 * @type {Object}
 */
const win = {
    /**
     * 从 localStorage 恢复 pin 状态并同步 alwaysOnTop
     * @returns {Promise<void>}
     */
    async restore() {
        pinned.value = localStorage.getItem(PIN_KEY) === 'true';
        await getCurrentWindow().setAlwaysOnTop(pinned.value);
    },

    /**
     * 切换置顶，同时持久化到 localStorage
     * @returns {Promise<void>}
     */
    async pin() {
        pinned.value = !pinned.value;
        localStorage.setItem(PIN_KEY, String(pinned.value));
        await getCurrentWindow().setAlwaysOnTop(pinned.value);
    },

    /**
     * 关闭窗口
     * @returns {Promise<void>}
     */
    async close() {
        speech.stop();
        pinned.value = false;
        localStorage.setItem(PIN_KEY, 'false');
        await getCurrentWindow().setAlwaysOnTop(false);
        await getCurrentWindow().hide();
    },

    /**
     * 失焦自动隐藏（置顶时跳过）
     * @returns {Promise<void>}
     */
    async on_blur() {
        if (pinned.value) return;
        const w = getCurrentWindow();
        setTimeout(async () => {
            if (pinned.value) return;
            try {
                const focused = await w.isFocused();
                if (!focused) {
                    speech.stop();
                    await w.hide();
                }
            } catch (_) {
                /* 窗口可能已销毁 */
            }
        }, 200);
    },

    /**
     * 拖拽窗口（无边框窗口需要手动调用）
     * @returns {Promise<void>}
     */
    async drag() {
        try {
            await getCurrentWindow().startDragging();
        } catch (_) {
            /* 忽略 */
        }
    },
};

/** @private 事件清理函数列表 */
const cleanup = [];

watch(original, (val) => {
    if (auto_speak.value && val) speech.original();
});

onMounted(async () => {
    await win.restore();
    const unlisten_translate = await listen('direct-translate', (event) => trans.run(event.payload.text, event.payload.source_app));
    cleanup.push(unlisten_translate);
    await emit('popup-ready');
    const w = getCurrentWindow();
    const unlisten_focus = await w.onFocusChanged(({ payload: focused }) => {
        if (!focused) win.on_blur();
    });
    cleanup.push(unlisten_focus);
});

onUnmounted(() => {
    cleanup.forEach((fn) => fn());
});
</script>

<template>
    <div class="flex h-screen flex-col">
        <div @mousedown="win.drag" class="border-color-border-2 bg-color-background-3 flex cursor-grab items-center justify-between border-b px-3 py-2">
            <span class="text-14 font-500 text-c-primary">划词翻译</span>
            <div class="flex items-center gap-1" @mousedown.stop>
                <Button
                    icon="pi pi-volume-up"
                    :severity="auto_speak ? 'warn' : 'secondary'"
                    text
                    rounded
                    size="small"
                    v-tooltip.bottom="'自动朗读原文'"
                    @click="
                        auto_speak = !auto_speak;
                        localStorage.setItem(AUTO_SPEAK_KEY, String(auto_speak));
                    "
                />
                <Button
                    icon="pi pi-thumbtack"
                    :severity="pinned ? 'warn' : 'secondary'"
                    text
                    rounded
                    size="small"
                    :style="{ transform: pinned ? 'none' : 'rotate(45deg)', transition: 'transform 0.2s ease' }"
                    v-tooltip.bottom="'固定窗口'"
                    @click="win.pin"
                />
                <Button icon="pi pi-times" severity="secondary" text rounded size="small" @click="win.close" />
            </div>
        </div>

        <div class="border-color-border-2 border-b p-3">
            <LangSelector
                v-model:source_lang="src_lang"
                v-model:target_lang="tgt_lang"
                @update:source_lang="trans.refresh"
                @update:target_lang="trans.refresh"
            />
        </div>

        <div class="flex-1 space-y-2 overflow-y-auto px-3 py-3">
            <div v-if="original" class="flex h-[21px] items-center gap-2 overflow-hidden px-1">
                <span class="bg-c-muted h-[14px] w-[2.5px] shrink-0 rounded-full"></span>
                <span class="text-14 text-c-muted truncate font-normal">{{ original }}</span>
                <svg
                    :class="[
                        'h-[16px] w-[16px] shrink-0 cursor-pointer transition-colors',
                        speech.is_active('original') ? 'text-c-primary' : 'text-c-muted hover:text-c-primary',
                        speech.fetching.value && speech.source.value === 'original' ? 'tts-pulse' : '',
                    ]"
                    @click="speech.original"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                    <path :class="{ 'tts-wave tts-wave-1': speech.playing.value && speech.source.value === 'original' }" d="M15.54 8.46a5 5 0 0 1 0 7.07" />
                    <path :class="{ 'tts-wave tts-wave-2': speech.playing.value && speech.source.value === 'original' }" d="M19.07 4.93a10 10 0 0 1 0 14.14" />
                </svg>
                <svg
                    class="text-c-muted hover:text-c-primary h-[14px] w-[14px] shrink-0 cursor-pointer"
                    @click="clip.original"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
            </div>

            <div v-if="loading" class="flex h-full w-full flex-col items-center justify-center gap-4">
                <ProgressSpinner style="width: 24px; height: 24px" stroke-width="3" />
                <span class="text-13 text-c-muted ml-2">翻译中…</span>
            </div>

            <Message v-if="error_msg" severity="error" :closable="false">{{ error_msg }}</Message>

            <div v-if="!loading && translation" class="markdown-body" v-html="rendered"></div>
        </div>

        <div class="border-color-border-3 bg-color-background-3 flex items-center justify-center gap-2 border-t px-3 py-2">
            <Button
                icon="pi pi-volume-up"
                label="朗读"
                :class="{
                    'tts-btn-playing': speech.playing.value && speech.source.value === 'result',
                    'tts-pulse': speech.fetching.value && speech.source.value === 'result',
                }"
                severity="secondary"
                text
                size="small"
                @click="speech.result"
            />
            <Button icon="pi pi-copy" label="复制" severity="secondary" text size="small" @click="clip.result" />
            <Button icon="pi pi-refresh" label="刷新" severity="secondary" text size="small" @click="trans.refresh" />
        </div>
    </div>
</template>
