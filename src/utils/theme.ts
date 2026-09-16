// 主题工具:CSS 变量 + localStorage。
// 抽出到独立文件,因为 <script setup> 不允许 ES module exports。

export type ThemeMode = 'system' | 'light' | 'dark';

export function applyTheme(mode: ThemeMode): void {
  localStorage.setItem('mmcode-theme', mode);
  const el = document.documentElement;
  el.classList.remove('theme-light', 'theme-dark');
  if (mode === 'light') {
    el.classList.add('theme-light');
  } else if (mode === 'dark') {
    el.classList.add('theme-dark');
  } else {
    // system:跟随系统
    const dark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    el.classList.add(dark ? 'theme-dark' : 'theme-light');
  }
}

export function readSavedTheme(): ThemeMode {
  const v = localStorage.getItem('mmcode-theme');
  if (v === 'light' || v === 'dark' || v === 'system') return v;
  return 'system';
}