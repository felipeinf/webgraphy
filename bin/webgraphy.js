#!/usr/bin/env node
import { execFileSync, spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "darwin") {
  console.error("webgraphy currently supports macOS only.");
  process.exit(1);
}

const pkgRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const appPath = path.join(pkgRoot, "Webgraphy.app");

if (!existsSync(appPath)) {
  console.error("Webgraphy.app is missing from this install.");
  process.exit(1);
}

try {
  execFileSync("xattr", ["-dr", "com.apple.quarantine", appPath], {
    stdio: "ignore",
  });
} catch {
}

const args = process.argv.slice(2);
const openArgs = args.length > 0 ? [appPath, "--args", ...args] : [appPath];
const child = spawn("open", openArgs, { stdio: "inherit" });

child.on("error", (err) => {
  console.error(err.message);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
