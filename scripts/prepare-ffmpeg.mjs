#!/usr/bin/env node
import { createRequire } from "node:module";
import { chmodSync, copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function hostTriple() {
  if (process.platform === "darwin") {
    return process.arch === "arm64" ? "aarch64-apple-darwin" : "x86_64-apple-darwin";
  }
  if (process.platform === "win32") {
    return process.arch === "arm64" ? "aarch64-pc-windows-msvc" : "x86_64-pc-windows-msvc";
  }
  if (process.platform === "linux") {
    return process.arch === "arm64" ? "aarch64-unknown-linux-gnu" : "x86_64-unknown-linux-gnu";
  }
  throw new Error(`unsupported host platform: ${process.platform}/${process.arch}`);
}

const target =
  process.env.SALADCART_TARGET_TRIPLE || process.env.TAURI_ENV_TARGET_TRIPLE || hostTriple();

if (target === "universal-apple-darwin") {
  throw new Error(
    "Universal macOS FFmpeg bundling is not configured yet. Build per architecture or add a lipo merge step.",
  );
}

let binary;
try {
  binary = require("@ffmpeg-installer/ffmpeg");
} catch (error) {
  throw new Error(
    `@ffmpeg-installer/ffmpeg is not available for this platform. Run pnpm install. ${error}`,
  );
}

const extension = target.includes("windows") ? ".exe" : "";
const outDir = path.join(root, "src-tauri", "binaries");
const destination = path.join(outDir, `ffmpeg-${target}${extension}`);
mkdirSync(outDir, { recursive: true });
copyFileSync(binary.path, destination);
if (!target.includes("windows")) chmodSync(destination, 0o755);
console.log(`bundled FFmpeg ${binary.version} -> ${path.relative(root, destination)}`);
