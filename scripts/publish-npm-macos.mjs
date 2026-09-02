import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const staging = path.join(root, ".npm-package");

if (process.platform !== "darwin") {
  console.error("This publish script only runs on macOS.");
  process.exit(1);
}

const cpu =
  process.arch === "arm64" ? "arm64" : process.arch === "x64" ? "x64" : null;
if (!cpu) {
  console.error(`Unsupported architecture: ${process.arch}`);
  process.exit(1);
}

function run(cmd, args, opts = {}) {
  const result = spawnSync(cmd, args, {
    stdio: "inherit",
    cwd: root,
    ...opts,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

if (process.env.WEBGRAPHY_SKIP_BUILD !== "1") {
  run("npx", ["tauri", "build", "--bundles", "app"]);
}

const appSrc = path.join(
  root,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "macos",
  "Webgraphy.app",
);
if (!existsSync(appSrc)) {
  console.error(`Missing app bundle at ${appSrc}`);
  process.exit(1);
}

const launcherSrc = path.join(root, "bin", "webgraphy.js");
const readmeSrc = path.join(root, "README.md");
const rootPkg = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));

rmSync(staging, { recursive: true, force: true });
mkdirSync(path.join(staging, "bin"), { recursive: true });
cpSync(appSrc, path.join(staging, "Webgraphy.app"), { recursive: true });
cpSync(launcherSrc, path.join(staging, "bin", "webgraphy.js"));
chmodSync(path.join(staging, "bin", "webgraphy.js"), 0o755);
if (existsSync(readmeSrc)) {
  cpSync(readmeSrc, path.join(staging, "README.md"));
}

writeFileSync(
  path.join(staging, "package.json"),
  `${JSON.stringify(
    {
      name: "webgraphy",
      version: rootPkg.version,
      description: rootPkg.description,
      type: "module",
      bin: {
        webgraphy: "bin/webgraphy.js",
      },
      os: ["darwin"],
      cpu: [cpu],
      engines: rootPkg.engines ?? { node: ">=18" },
      files: ["bin", "Webgraphy.app"],
      keywords: rootPkg.keywords ?? [],
      author: rootPkg.author ?? "felipeinf",
      license: rootPkg.license ?? "MIT",
      repository: rootPkg.repository,
      bugs: rootPkg.bugs,
      homepage: rootPkg.homepage,
      publishConfig: {
        access: "public",
      },
    },
    null,
    2,
  )}\n`,
);

run("npm", ["publish", "--access", "public"], { cwd: staging });
