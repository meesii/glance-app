import { PrimeVueResolver } from '@primevue/auto-import-resolver';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import components from 'unplugin-vue-components/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
    resolve: {
        alias: {
            '@': path.resolve(__dirname, 'src'),
        },
    },
    plugins: [
        vue(),
        tailwindcss(),
        components({
            resolvers: [PrimeVueResolver()],
        }),
    ],
    build: {
        rollupOptions: {
            input: {
                index: path.resolve(__dirname, 'index.html'),
                settings: path.resolve(__dirname, 'settings.html'),
            },
        },
    },
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
        watch: {
            ignored: ['**/src-tauri/**'],
        },
    },
}));
