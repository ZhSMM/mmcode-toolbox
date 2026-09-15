import { createApp } from 'vue';
import { createPinia } from 'pinia';
import ElementPlus from 'element-plus';
import zhCn from 'element-plus/dist/locale/zh-cn.mjs';
import * as ElementPlusIconsVue from '@element-plus/icons-vue';

import App from './App.vue';
import { router } from './router';

import './styles/global.css';

const app = createApp(App);

// 注册全部 Element Plus 图标（按需懒加载也行,这里全量简单直接）
for (const [name, comp] of Object.entries(ElementPlusIconsVue)) {
  app.component(name, comp as any);
}

app.use(createPinia());
app.use(router);
app.use(ElementPlus, { locale: zhCn });

app.mount('#app');