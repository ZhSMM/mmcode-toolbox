/**
 * Tauri invoke 统一封装。
 *
 * 作用:
 *  1. 给 args 添加统一的 _type 头(可选,用于后端日志);
 *  2. 把 AppError 序列化的 { kind, message } 转换为 Error,方便上层 try/catch;
 *  3. 自动记一条 console.log(开发态可观察)。
 */
import { invoke } from '@tauri-apps/api/core';

export class ApiError extends Error {
  kind: string;
  constructor(kind: string, message: string) {
    super(message);
    this.kind = kind;
    this.name = 'ApiError';
  }
}

export async function invokeCmd<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (import.meta.env.DEV) {
    // eslint-disable-next-line no-console
    console.debug(`[invoke] ${cmd}`, args);
  }
  try {
    return await invoke<T>(cmd, args);
  } catch (e: unknown) {
    // Rust 端 AppError 序列化为 { kind, message }
    const obj = e as { kind?: string; message?: string };
    const kind = obj?.kind ?? 'unknown';
    const message = obj?.message ?? String(e);
    throw new ApiError(kind, message);
  }
}