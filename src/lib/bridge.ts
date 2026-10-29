import { convertFileSrc, invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";
import { mockInvoke, mockListen } from "./mock";

export const isTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

export const isDesktop = isTauri;

export async function pickAssetPaths(): Promise<string[] | null> {
  if (!isTauri) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    multiple: true,
    title: "导入素材",
    filters: [
      {
        name: "媒体",
        extensions: [
          "png", "jpg", "jpeg", "webp",
          "mp4", "mov", "webm",
          "mp3", "wav", "m4a", "aac", "flac",
        ],
      },
    ],
  });
  if (selected === null) return [];
  return Array.isArray(selected) ? selected : [selected];
}

export async function pickNovelFile(): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    multiple: false,
    title: "导入小说",
    filters: [{ name: "小说文本", extensions: ["txt", "md", "text"] }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function pickDirectory(title: string): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, multiple: false, title });
  return typeof selected === "string" ? selected : null;
}

export async function invoke<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (isTauri) return tauriInvoke<T>(command, args);
  return mockInvoke<T>(command, args);
}

export async function listen<T>(
  event: string,
  callback: (event: { payload: T }) => void,
): Promise<UnlistenFn> {
  if (isTauri) return tauriListen<T>(event, callback);
  return mockListen<T>(event, callback);
}

export function assetUrl(path: string | null | undefined): string {
  if (!path) return "";
  if (path.startsWith("data:") || path.startsWith("blob:") || path.startsWith("http")) {
    return path;
  }
  // macOS/Linux project assets are absolute paths beginning with `/`.
  // They must go through the Tauri asset protocol, not be treated as web roots.
  return isTauri ? convertFileSrc(path) : path;
}

export function formatError(error: unknown): string {
  if (typeof error === "string") {
    try {
      const parsed = JSON.parse(error) as { message?: string };
      if (parsed?.message) return parsed.message;
    } catch {
      // fall through
    }
    return error;
  }
  if (error && typeof error === "object") {
    const value = error as { message?: string; code?: string };
    if (value.message) return value.message;
    if (value.code) return `操作失败 (${value.code})`;
  }
  return "操作失败，请稍后重试。";
}

export function formatCurrency(value: number): string {
  if (!value) return "¥0.00";
  return `¥${value.toFixed(2)}`;
}

export function formatDuration(ms: number | null | undefined): string {
  if (!ms) return "0s";
  const total = Math.round(ms / 1000);
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return minutes > 0 ? `${minutes}:${String(seconds).padStart(2, "0")}` : `${seconds}s`;
}

export function formatTimecode(ms: number): string {
  const fps = 30;
  const totalFrames = Math.round((Math.max(0, ms) * fps) / 1000);
  const minutes = Math.floor(totalFrames / (fps * 60));
  const seconds = Math.floor(totalFrames / fps) % 60;
  const frames = totalFrames % fps;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}:${String(frames).padStart(2, "0")}`;
}

export function relativeTime(value: string): string {
  const then = new Date(value).getTime();
  if (Number.isNaN(then)) return "";
  const diff = Date.now() - then;
  const minutes = Math.round(diff / 60000);
  if (minutes < 1) return "刚刚";
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.round(hours / 24);
  if (days < 30) return `${days} 天前`;
  return new Date(value).toLocaleDateString("zh-CN");
}
