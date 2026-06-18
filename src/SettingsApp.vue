<script setup>
import { onMounted, ref } from 'vue';
import { get_chrome_cookies, get_config, save_config, update_mouse_hook, update_shortcuts } from './shared/api.js';
import message from './utils/message.js';

const active_tab = ref(0);
const doubao_cookie = ref('');
const doubao_model = ref('Doubao');
const shortcut_selection = ref('');
const middle_click_trigger = ref(false);
const cookie_loading = ref(false);

const tts_provider = ref('Doubao');
const mimo_api_key = ref('');
const mimo_base_url = ref('https://api.xiaomimimo.com/v1');
const mimo_voice = ref('冰糖');
const mimo_model = ref('mimo-v2.5-tts');
const mimo_voice_design = ref('');

const tts_provider_options = [
    { label: '豆包', value: 'Doubao' },
    { label: '小米 MiMo', value: 'Mimo' },
];

const mimo_model_options = [
    { label: 'mimo-v2.5-tts', value: 'mimo-v2.5-tts' },
    { label: 'mimo-v2.5-tts-voicedesign', value: 'mimo-v2.5-tts-voicedesign' },
];

const mimo_voice_options = [
    { label: '冰糖（中文·女）', value: '冰糖' },
    { label: '茉莉（中文·女）', value: '茉莉' },
    { label: '苏打（中文·男）', value: '苏打' },
    { label: '白桦（中文·男）', value: '白桦' },
    { label: 'Mia（英文·女）', value: 'Mia' },
    { label: 'Chloe（英文·女）', value: 'Chloe' },
    { label: 'Milo（英文·男）', value: 'Milo' },
    { label: 'Dean（英文·男）', value: 'Dean' },
];

const model_options = [
    { label: '豆包', value: 'Doubao' },
    { label: '火山', value: 'Volcano' },
    { label: '微软', value: 'Microsoft' },
];

async function fetch_chrome_cookies() {
    cookie_loading.value = true;
    try {
        const cookies = await get_chrome_cookies();
        doubao_cookie.value = cookies;
        const config = await get_config();
        config.doubao_cookie = cookies;
        await save_config(config);
        message.success('已获取并保存 Cookie');
    } catch (err) {
        message.error(err.message);
    } finally {
        cookie_loading.value = false;
    }
}

async function save() {
    const config = {
        doubao_cookie: doubao_cookie.value || null,
        doubao_model: doubao_model.value,
        shortcut_selection: shortcut_selection.value,
        middle_click_trigger: middle_click_trigger.value,
        tts: {
            provider: tts_provider.value,
            mimo_api_key: mimo_api_key.value || null,
            mimo_base_url: mimo_base_url.value,
            mimo_voice: mimo_voice.value,
            mimo_model: mimo_model.value,
            mimo_voice_design: mimo_voice_design.value,
        },
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
        if (config.tts) {
            tts_provider.value = config.tts.provider || 'Doubao';
            mimo_api_key.value = config.tts.mimo_api_key || '';
            mimo_base_url.value = config.tts.mimo_base_url || 'https://api.xiaomimimo.com/v1';
            mimo_voice.value = config.tts.mimo_voice || '冰糖';
            mimo_model.value = config.tts.mimo_model || 'mimo-v2.5-tts';
            mimo_voice_design.value = config.tts.mimo_voice_design || '';
        }
    } catch (err) {
        message.error(`加载配置失败：${err.message}`);
    }
});
</script>

<template>
    <div class="bg-color-background-2 flex h-full flex-col p-4">
        <Tabs v-model:value="active_tab" class="border-color-border-2 bg-color-background-1 flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border">
            <TabList>
                <Tab :value="0"> <i class="pi pi-language text-13 mr-1.5" />翻译 </Tab>
                <Tab :value="1"> <i class="pi pi-volume-up text-13 mr-1.5" />朗读 </Tab>
            </TabList>
            <TabPanels class="min-h-0 flex-1 overflow-y-auto p-4">
                <TabPanel :value="0" class="space-y-3 pt-3">
                    <div class="flex flex-col gap-2">
                        <label class="text-13 text-c-secondary font-medium">豆包 Cookie</label>
                        <Textarea v-model="doubao_cookie" placeholder="请输入豆包 Cookie" rows="3" class="text-13 w-full resize-none" />
                        <div class="flex items-center gap-2">
                            <Button
                                label="从 Chrome 获取"
                                icon="pi pi-refresh"
                                size="small"
                                :loading="cookie_loading"
                                variant="outlined"
                                @click="fetch_chrome_cookies"
                            />
                            <span class="text-12 text-c-muted">从浏览器开发者工具中复制</span>
                        </div>
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-13 text-c-secondary font-medium">翻译模型</label>
                        <Select v-model="doubao_model" :options="model_options" option-label="label" option-value="value" class="w-full" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-13 text-c-secondary font-medium">快捷键</label>
                        <InputText v-model="shortcut_selection" placeholder="Alt+E" class="w-full" />
                        <span class="text-12 text-c-muted">选中文字后按此快捷键直接翻译</span>
                    </div>
                    <div class="flex items-center justify-between">
                        <div class="flex flex-col gap-0.5">
                            <label class="text-13 text-c-secondary font-medium">鼠标中键翻译</label>
                            <span class="text-12 text-c-muted">选中文字后按鼠标中键翻译</span>
                        </div>
                        <ToggleSwitch v-model="middle_click_trigger" />
                    </div>
                </TabPanel>

                <TabPanel :value="1" class="space-y-3 pt-3">
                    <div class="flex flex-col gap-2">
                        <label class="text-13 text-c-secondary font-medium">TTS 提供商</label>
                        <Select v-model="tts_provider" :options="tts_provider_options" option-label="label" option-value="value" class="w-full" />
                    </div>
                    <template v-if="tts_provider === 'Doubao'">
                        <div class="flex flex-col gap-2">
                            <label class="text-13 text-c-secondary font-medium">豆包 Cookie</label>
                            <Textarea v-model="doubao_cookie" placeholder="请输入豆包 Cookie" rows="3" class="text-13 w-full resize-none" />
                            <div class="flex items-center gap-2">
                                <Button
                                    label="从 Chrome 获取"
                                    icon="pi pi-refresh"
                                    size="small"
                                    :loading="cookie_loading"
                                    variant="outlined"
                                    @click="fetch_chrome_cookies"
                                />
                                <span class="text-12 text-c-muted">从浏览器开发者工具中复制</span>
                            </div>
                        </div>
                    </template>
                    <template v-if="tts_provider === 'Mimo'">
                        <div class="flex flex-col gap-2">
                            <label class="text-13 text-c-secondary font-medium">模型</label>
                            <Select v-model="mimo_model" :options="mimo_model_options" option-label="label" option-value="value" class="w-full" />
                        </div>
                        <template v-if="mimo_model === 'mimo-v2.5-tts-voicedesign'">
                            <div class="flex flex-col gap-2">
                                <label class="text-13 text-c-secondary font-medium">音色描述</label>
                                <Textarea
                                    v-model="mimo_voice_design"
                                    placeholder="如：温柔治愈系女声，语速缓慢，带有深夜电台的感觉"
                                    rows="3"
                                    class="text-13 w-full resize-none"
                                />
                                <span class="text-12 text-c-muted">描述目标音色特征，不填则默认「标准播音腔，声音清晰自然」</span>
                            </div>
                        </template>
                        <template v-else>
                            <div class="flex flex-col gap-2">
                                <label class="text-13 text-c-secondary font-medium">语音角色</label>
                                <Select v-model="mimo_voice" :options="mimo_voice_options" option-label="label" option-value="value" class="w-full" />
                            </div>
                        </template>
                        <div class="flex flex-col gap-2">
                            <label class="text-13 text-c-secondary font-medium">API Key</label>
                            <Password v-model="mimo_api_key" placeholder="请输入 MiMo API Key" class="w-full" :feedback="false" toggle-mask />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="text-13 text-c-secondary font-medium">Base URL</label>
                            <InputText v-model="mimo_base_url" placeholder="https://api.xiaomimimo.com/v1" class="w-full" />
                        </div>
                    </template>
                </TabPanel>
            </TabPanels>
        </Tabs>

        <Button label="保存" icon="pi pi-check" class="mt-3 w-full" @click="save" />

        <Toast
            style="top: 16px"
            position="top-center"
            :dt="{
                icon: { size: '16px' },
                border: { radius: '16px' },
                content: { padding: '12px 16px', gap: '6px' },
                summary: { font: { weight: '500', size: '13px' } },
                close: { button: { width: '14px', height: '14px' } },
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
