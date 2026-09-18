/**
 * Tauri invoke 统一封装。
 *
 * 设计目标:
 * 1. 失败时打 console.error(完整上下文: cmd 名、参数、错误 kind/message/栈),
 *    方便从 devtools 调试
 * 2. 成功时 INFO 级别摘要(可在 DevTools Network-like 面板看到调用流)
 * 3. 归一化错误为 ApiError,UI 层可以 try/catch
 */
import { invoke } from '@tauri-apps/api/core';

export class ApiError extends Error {
  kind: string;
  retryAfter?: number;
  raw: unknown;

  constructor(kind: string, message: string, raw: unknown, retryAfter?: number) {
    super(message);
    this.kind = kind;
    this.retryAfter = retryAfter;
    this.raw = raw;
    this.name = 'ApiError';
  }
}

function previewArgs(args: Record<string, unknown> | undefined): string {
  if (!args) return '';
  try {
    const seen = new WeakSet();
    const json = JSON.stringify(args, (_k, v) => {
      if (typeof v === 'string' && v.length > 100) return v.slice(0, 100) + '…(+' + (v.length - 100) + ' chars)';
      if (typeof v === 'object' && v !== null) {
        if (seen.has(v)) return '[Circular]';
        seen.add(v);
      }
      return v;
    });
    return json.length > 300 ? json.slice(0, 300) + '…' : json;
  } catch {
    return '[unserializable]';
  }
}

function fmtTime(ms: number): string {
  return ms < 1 ? '<1ms' : ms < 1000 ? `${ms.toFixed(1)}ms` : `${(ms / 1000).toFixed(2)}s`;
}

export async function invokeCmd<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const start = performance.now();
  if (import.meta.env.DEV) {
    console.debug(`[invoke] ▶ ${cmd}`, args ? previewArgs(args) : '');
  }
  try {
    const result = await invoke<T>(cmd, args);
    const ms = performance.now() - start;
    if (import.meta.env.DEV) {
      console.debug(`[invoke] ✓ ${cmd} (${fmtTime(ms)})`);
    }
    return result;
  } catch (e: unknown) {
    const ms = performance.now() - start;
    const obj = e as { kind?: string; message?: string; retryAfter?: number };
    const kind = obj?.kind ?? 'unknown';
    const message = obj?.message ?? String(e);
    const retryAfter = obj?.retryAfter;

    // 详细日志:dev 模式 console.error(可展开),prod 也保留以防白屏
    console.error(
      `[invoke] ✗ ${cmd} (${fmtTime(ms)})\n` +
      `  kind:        ${kind}\n` +
      `  message:     ${message}\n` +
      `  args:        ${args ? previewArgs(args) : '(none)'}\n` +
      (retryAfter !== undefined ? `  retryAfter:  ${retryAfter}s\n` : '') +
      `  stack:       ${(e as Error)?.stack ?? '(no stack)'}`,
    );

    throw new ApiError(kind, message, e, retryAfter);
  }
}