import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import MainLayout from '@/components/layout/MainLayout.vue';
import Home from '@/views/Home.vue';
import JsonFormatter from '@/components/tools/JsonFormatter.vue';
import Base64Tool from '@/components/tools/Base64Tool.vue';
import PlaceholderTool from '@/components/common/PlaceholderTool.vue';
import Settings from '@/views/Settings.vue';

// 占位工具列表：除了已实现的 JSON / Base64 外，其他都用 PlaceholderTool 渲染
const PLACEHOLDER_ROUTES: RouteRecordRaw[] = [
  { path: '/tools/url-codec',    component: PlaceholderTool, props: { toolId: 'url-codec',    title: 'URL 编解码' } },
  { path: '/tools/hex',          component: PlaceholderTool, props: { toolId: 'hex',          title: 'Hex 转换' } },
  { path: '/tools/regex-tester', component: PlaceholderTool, props: { toolId: 'regex-tester', title: '正则测试' } },
  { path: '/tools/diff',         component: PlaceholderTool, props: { toolId: 'diff',         title: 'Diff 对比' } },
  { path: '/tools/string-stats', component: PlaceholderTool, props: { toolId: 'string-stats', title: '字符串统计' } },
  { path: '/tools/md5',          component: PlaceholderTool, props: { toolId: 'md5',          title: 'MD5' } },
  { path: '/tools/sha',          component: PlaceholderTool, props: { toolId: 'sha',          title: 'SHA-1/256' } },
  { path: '/tools/hmac',         component: PlaceholderTool, props: { toolId: 'hmac',         title: 'HMAC' } },
  { path: '/tools/uuid',         component: PlaceholderTool, props: { toolId: 'uuid',         title: 'UUID 生成' } },
  { path: '/tools/password',     component: PlaceholderTool, props: { toolId: 'password',     title: '随机密码' } },
  { path: '/tools/timestamp',    component: PlaceholderTool, props: { toolId: 'timestamp',    title: '时间戳转换' } },
  { path: '/tools/qrcode',       component: PlaceholderTool, props: { toolId: 'qrcode',       title: '二维码生成' } },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      redirect: '/home',
      children: [
        { path: 'home', name: 'home', component: Home, meta: { title: '首页' } },
        { path: 'settings', name: 'settings', component: Settings, meta: { title: '设置' } },

        // 已实现的工具
        { path: 'tools/json-formatter', name: 'json-formatter', component: JsonFormatter, meta: { title: 'JSON 格式化' } },
        { path: 'tools/base64',         name: 'base64',         component: Base64Tool,    meta: { title: 'Base64 编解码' } },

        // 占位工具
        ...PLACEHOLDER_ROUTES,
      ],
    },
  ],
});

router.afterEach((to) => {
  const title = (to.meta?.title as string) || 'Mavis Code Toolbox';
  document.title = `${title} · Mavis Code Toolbox`;
});