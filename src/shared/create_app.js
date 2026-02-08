import '@/assets/styles/base.css';
import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';
import PrimeVue from 'primevue/config';
import Tooltip from 'primevue/tooltip';
import { createApp } from 'vue';

const Fluent = definePreset(Aura, {
    semantic: {
        colorScheme: {
            light: {
                primary: {
                    color: '{neutral.950}',
                    inverseColor: '#ffffff',
                    hoverColor: '{neutral.900}',
                    activeColor: '{neutral.800}',
                },
                highlight: {
                    background: '{neutral.950}',
                    focusBackground: '{neutral.700}',
                    color: '#ffffff',
                    focusColor: '#ffffff',
                },
            },
        },
    },
});

/**
 * 创建并挂载 Vue 应用（所有窗口共用）
 * @param {import('vue').Component} root_component - 根组件
 * @returns {import('vue').App}
 */
export function create_app(root_component) {
    const app = createApp(root_component);
    app.use(PrimeVue, {
        unstyled: false,
        ripple: false,
        theme: {
            preset: Fluent,
            options: {
                darkModeSelector: false,
                cssLayer: {
                    name: 'primevue',
                    order: 'theme, base, primevue',
                },
            },
        },
    });
    app.directive('tooltip', Tooltip);
    app.mount('#app');
    return app;
}
