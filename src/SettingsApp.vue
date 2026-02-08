<script setup>
import { onMounted, ref } from 'vue';
import { get_config, save_config, update_mouse_hook, update_shortcuts } from './shared/api.js';
import message from './utils/message.js';

const doubao_cookie = ref('');
const doubao_model = ref('Doubao');
const shortcut_selection = ref('');
const middle_click_trigger = ref(false);

const model_options = [
    { label: '豆包', value: 'Doubao' },
    { label: '火山', value: 'Volcano' },
    { label: '微软', value: 'Microsoft' },
];

async function save() {
    const config = {
        doubao_cookie: doubao_cookie.value || null,
        doubao_model: doubao_model.value,
        shortcut_selection: shortcut_selection.value,
        middle_click_trigger: middle_click_trigger.value,
    };

    try {
        await save_config(config);
    } catch (err) {
        message.error(`保存失败：${err.message}`);
        return;
    }

    try {
        await update_shortcuts(config);
    } catch (_) {
        /* 静默忽略 */
    }
    try {
        await update_mouse_hook(config.middle_click_trigger);
    } catch (_) {
        /* 静默忽略 */
    }
    message.success('保存成功');
}

onMounted(async () => {
    try {
        const config = await get_config();
        doubao_cookie.value = config.doubao_cookie || '';
        doubao_model.value = config.doubao_model || 'Doubao';
        shortcut_selection.value = config.shortcut_selection || '';
        middle_click_trigger.value = config.middle_click_trigger || false;
    } catch (err) {
        message.error(`加载配置失败：${err.message}`);
    }
});
</script>

<template>
    <div class="bg-color-background-2 h-full">
        <div class="mx-auto flex h-full flex-col gap-2 p-6">
            <div class="text-18 text-c-primary font-semibold">翻译设置</div>

            <div class="border-color-border-2 bg-color-background-1 flex-1 space-y-5 rounded-xl border p-5">
                <div class="flex flex-col gap-1.5">
                    <label class="text-13 text-c-secondary font-medium">豆包 Cookie</label>
                    <Password v-model="doubao_cookie" placeholder="请输入豆包 Cookie" :feedback="false" toggle-mask class="w-full" input-class="w-full" />
                    <span class="text-11 text-c-muted"> 从浏览器开发者工具中复制 Cookie </span>
                </div>

                <div class="flex flex-col gap-1.5">
                    <label class="text-13 text-c-secondary font-medium">翻译模型</label>
                    <Select v-model="doubao_model" :options="model_options" option-label="label" option-value="value" class="w-full" />
                </div>

                <div class="flex flex-col gap-1.5">
                    <label class="text-13 text-c-secondary font-medium"> 快捷键 </label>
                    <InputText v-model="shortcut_selection" placeholder="Alt+E" class="w-full" />
                    <span class="text-11 text-c-muted"> 选中文字后按此快捷键直接翻译 </span>
                </div>

                <div class="flex items-center justify-between">
                    <div class="flex flex-col gap-0.5">
                        <label class="text-13 text-c-secondary font-medium">鼠标中键翻译</label>
                        <span class="text-11 text-c-muted">选中文字后按鼠标中键直接翻译</span>
                    </div>
                    <ToggleSwitch v-model="middle_click_trigger" />
                </div>

                <Button label="保存" icon="pi pi-check" class="w-full" @click="save" />
            </div>
        </div>

        <Toast
            style="top: 16px"
            position="top-center"
            :dt="{
                icon: {
                    size: '16px',
                },
                border: {
                    radius: '16px',
                },
                content: {
                    padding: '12px 16px',
                    gap: '6px',
                },
                summary: {
                    font: {
                        weight: '500',
                        size: '13px',
                    },
                },
                close: {
                    button: {
                        width: '14px',
                        height: '14px',
                    },
                },
            }"
            :pt="{
                root: 'w-fit',
                message: 'bg-white border-color-border-3 shadow-toast m-0 rounded-xl min-w-[120px] flex items-center justify-center',
                messageContent: 'items-center gap-2',
                messageIcon: 'h-4 w-4',
                closeButton: 'm-0 right-0',
                summary: 'text-c-primary text-12',
            }"
        />
    </div>
</template>
