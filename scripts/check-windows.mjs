#!/usr/bin/env node
// Compile-only Windows gate. On macOS/Linux this uses cargo-xwin; the resource
// compiler shim exists only because `cargo check` still runs tauri-build's
// Windows resource step and this host has no llvm-rc. It never produces a
// bundle and must not be used for release builds.
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = "x86_64-pc-windows-msvc";
const cargo = process.env.CARGO || "cargo";
const sidecar = path.join(root, "src-tauri", "binaries", `ffmpeg-${target}.exe`);

let placeholder = false;
let shimDir;
if (!existsSync(sidecar)) {
  mkdirSync(path.dirname(sidecar), { recursive: true });
  writeFileSync(sidecar, "");
  placeholder = true;
}

try {
  let args;
  if (process.platform === "win32") {
    args = ["check", "--target", target];
  } else {
    const rc = spawnSync("llvm-rc", ["-V"], { encoding: "utf8" });
    if (rc.error || rc.status !== 0) {
      shimDir = mkdtempSync(path.join(os.tmpdir(), "saladcart-rc-"));
      const shim = path.join(shimDir, "llvm-rc");
      writeFileSync(
        shim,
        `#!/bin/sh
if [ "$1" = "-V" ]; then
  echo "LLVM Resource Converter (check shim) no-preprocess"
  exit 0
fi
out=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "/fo" ]; then out="$arg"; fi
  prev="$arg"
done
if [ -n "$out" ]; then printf 'CHECK' > "$out"; fi
exit 0
`,
      );
      chmodSync(shim, 0o755);
    }
    const probe = spawnSync(cargo, ["xwin", "--version"], { encoding: "utf8" });
    if (probe.status !== 0) {
      console.error("cargo-xwin is required: cargo install cargo-xwin");
      process.exit(1);
    }
    args = ["xwin", "check", "--target", target];
  }

  const env = { ...process.env };
  if (shimDir) env.PATH = `${shimDir}${path.delimiter}${env.PATH ?? ""}`;
  const result = spawnSync(cargo, args, {
    cwd: path.join(root, "src-tauri"),
    env,
    stdio: "inherit",
  });
  process.exitCode = result.status ?? 1;
} finally {
  if (placeholder) rmSync(sidecar, { force: true });
  if (shimDir) rmSync(shimDir, { recursive: true, force: true });
}
