import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import MainLayout from '@/components/layout/MainLayout.vue';
import Home from '@/views/Home.vue';
import JsonFormatter from '@/components/tools/JsonFormatter.vue';
import Base64Tool from '@/components/tools/Base64Tool.vue';
import UrlCodecTool from '@/components/tools/UrlCodecTool.vue';
import HexTool from '@/components/tools/HexTool.vue';
import RegexTester from '@/components/tools/RegexTester.vue';
import DiffTool from '@/components/tools/DiffTool.vue';
import StringStatsTool from '@/components/tools/StringStatsTool.vue';
import Md5Tool from '@/components/tools/Md5Tool.vue';
import ShaTool from '@/components/tools/ShaTool.vue';
import HmacTool from '@/components/tools/HmacTool.vue';
import UuidTool from '@/components/tools/UuidTool.vue';
import PasswordTool from '@/components/tools/PasswordTool.vue';
import TimestampTool from '@/components/tools/TimestampTool.vue';
import QrcodeTool from '@/components/tools/QrcodeTool.vue';
import AesTool from '@/components/tools/AesTool.vue';
import JwtTool from '@/components/tools/JwtTool.vue';
import MarkdownTool from '@/components/tools/MarkdownTool.vue';
import CsvTool from '@/components/tools/CsvTool.vue';
import ColorTool from '@/components/tools/ColorTool.vue';
import CronTool from '@/components/tools/CronTool.vue';
import SqlFormatTool from '@/components/tools/SqlFormatTool.vue';
import BaseNTool from '@/components/tools/BaseNTool.vue';
import CidrTool from '@/components/tools/CidrTool.vue';
import IpInfoTool from '@/components/tools/IpInfoTool.vue';
import RandomTool from '@/components/tools/RandomTool.vue';
import Settings from '@/views/Settings.vue';

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

        { path: 'tools/json-formatter', name: 'json-formatter', component: JsonFormatter, meta: { title: 'JSON 格式化' } },
        { path: 'tools/base64',         name: 'base64',         component: Base64Tool,    meta: { title: 'Base64 编解码' } },
        { path: 'tools/url-codec',      name: 'url-codec',      component: UrlCodecTool,   meta: { title: 'URL 编解码' } },
        { path: 'tools/hex',            name: 'hex',            component: HexTool,        meta: { title: 'Hex 转换' } },
        { path: 'tools/regex-tester',   name: 'regex-tester',   component: RegexTester,    meta: { title: '正则测试' } },
        { path: 'tools/diff',           name: 'diff',           component: DiffTool,       meta: { title: 'Diff 对比' } },
        { path: 'tools/string-stats',   name: 'string-stats',   component: StringStatsTool, meta: { title: '字符串统计' } },
        { path: 'tools/md5',            name: 'md5',            component: Md5Tool,        meta: { title: 'MD5' } },
        { path: 'tools/sha',            name: 'sha',            component: ShaTool,        meta: { title: 'SHA-1/256/512' } },
        { path: 'tools/hmac',           name: 'hmac',           component: HmacTool,       meta: { title: 'HMAC' } },
        { path: 'tools/uuid',           name: 'uuid',           component: UuidTool,       meta: { title: 'UUID 生成' } },
        { path: 'tools/password',       name: 'password',       component: PasswordTool,   meta: { title: '随机密码' } },
        { path: 'tools/timestamp',      name: 'timestamp',      component: TimestampTool,  meta: { title: '时间戳转换' } },
        { path: 'tools/qrcode',         name: 'qrcode',         component: QrcodeTool,     meta: { title: '二维码生成' } },
        { path: 'tools/aes',            name: 'aes',            component: AesTool,        meta: { title: 'AES-256-GCM' } },
        { path: 'tools/jwt',            name: 'jwt',            component: JwtTool,        meta: { title: 'JWT 解码' } },
        { path: 'tools/markdown',       name: 'markdown',       component: MarkdownTool,   meta: { title: 'Markdown 预览' } },
        { path: 'tools/csv-viewer',     name: 'csv-viewer',     component: CsvTool,        meta: { title: 'CSV 工具' } },
        { path: 'tools/color',          name: 'color',          component: ColorTool,      meta: { title: '颜色转换' } },
        { path: 'tools/cron',           name: 'cron',           component: CronTool,       meta: { title: 'Cron 解析' } },
        { path: 'tools/sql-format',     name: 'sql-format',     component: SqlFormatTool,  meta: { title: 'SQL 格式化' } },
        { path: 'tools/base-n',         name: 'base-n',         component: BaseNTool,      meta: { title: 'Base32 / Base58' } },
        { path: 'tools/cidr',           name: 'cidr',           component: CidrTool,       meta: { title: 'CIDR 子网' } },
        { path: 'tools/ip-info',        name: 'ip-info',        component: IpInfoTool,     meta: { title: 'IP 信息' } },
        { path: 'tools/random',         name: 'random',         component: RandomTool,     meta: { title: '随机数 / 抽样' } },
      ],
    },
  ],
});

router.afterEach((to) => {
  const title = (to.meta?.title as string) || 'Mavis Code Toolbox';
  document.title = `${title} · Mavis Code Toolbox`;
});