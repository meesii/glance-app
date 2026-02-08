<script setup>
import { LANGUAGES, target_langs } from '@/shared/i18n.js';

const source_lang = defineModel('source_lang', { type: String, default: 'auto' });
const target_lang = defineModel('target_lang', { type: String, default: 'zh' });

const source_options = LANGUAGES;
const target_options = target_langs();
</script>

<template>
    <div class="flex items-center gap-2">
        <Select v-model="source_lang" :options="source_options" option-label="label" option-value="code" class="flex-1" size="small" />
        <i class="pi pi-arrow-right text-12 text-c-muted" />
        <Select v-model="target_lang" :options="target_options" option-label="label" option-value="code" class="flex-1" size="small" />
    </div>
</template>
