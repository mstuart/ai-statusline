import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const targetByPlatform = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const run = (command, args, options = {}) =>
  new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: "inherit", ...options });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(new Error(`${command} ${args.join(" ")} exited with ${code}`));
    });
  });

const platformKey = `${process.platform}-${process.arch}`;
const target = targetByPlatform[platformKey];
const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageJson = JSON.parse(
  fs.readFileSync(new URL("../package.json", import.meta.url), "utf8")
);

if (!target) {
  throw new Error(`Unsupported smoke-test platform: ${platformKey}`);
}

const extension = target.includes("windows") ? "zip" : "tar.gz";
const archiveName = `ai-statusline-v${packageJson.version}-${target}.${extension}`;
const repoRoot = path.resolve(scriptDir, "..", "..");
const scratchDir = fs.mkdtempSync(
  path.join(os.tmpdir(), "ai-statusline-pack-smoke-")
);
const fixtureDir = path.join(scratchDir, "fixtures");
const prefixDir = path.join(scratchDir, "prefix");
const distDir = path.join(scratchDir, "dist");
const sourceBinary = path.join(
  repoRoot,
  "target",
  "release",
  process.platform === "win32" ? "ai-statusline.exe" : "ai-statusline"
);
const stagedBinary = path.join(
  distDir,
  process.platform === "win32" ? "ai-statusline-bin.exe" : "ai-statusline-bin"
);

if (!fs.existsSync(sourceBinary)) {
  throw new Error(
    `Missing release binary at ${sourceBinary}. Run cargo build --release first.`
  );
}

fs.mkdirSync(fixtureDir, { recursive: true });
fs.mkdirSync(distDir, { recursive: true });
fs.copyFileSync(sourceBinary, stagedBinary);
if (process.platform === "win32") {
  execFileSync("powershell", [
    "-NoProfile",
    "-Command",
    `Compress-Archive -Path ${JSON.stringify(stagedBinary)} -DestinationPath ${JSON.stringify(path.join(fixtureDir, archiveName))}`,
  ]);
} else {
  fs.chmodSync(stagedBinary, 0o755);
  execFileSync("tar", [
    "-C",
    distDir,
    "-czf",
    path.join(fixtureDir, archiveName),
    "ai-statusline-bin",
  ]);
}

const server = http.createServer((request, response) => {
  const requested = decodeURIComponent(
    new URL(request.url, "http://localhost").pathname.slice(1)
  );
  if (requested !== archiveName) {
    response.writeHead(404);
    response.end("not found");
    return;
  }

  response.writeHead(200, { "content-type": "application/octet-stream" });
  fs.createReadStream(path.join(fixtureDir, archiveName)).pipe(response);
});

try {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  const packOutput = execFileSync("npm", ["pack", "--silent"], {
    cwd: path.resolve(scriptDir, ".."),
    encoding: "utf8",
  }).trim();
  const tarballPath = path.resolve(
    scriptDir,
    "..",
    packOutput.split("\n").at(-1)
  );

  try {
    await run("npm", ["install", "--prefix", prefixDir, tarballPath], {
      env: {
        ...process.env,
        AI_STATUSLINE_DOWNLOAD_BASE_URL: `http://127.0.0.1:${port}`,
      },
    });

    const packageDir = path.join(prefixDir, "node_modules", packageJson.name);
    const expectedBinary = path.join(
      packageDir,
      "bin",
      process.platform === "win32"
        ? "ai-statusline-bin.exe"
        : "ai-statusline-bin"
    );
    fs.rmSync(expectedBinary, { force: true });
    await run("node", [path.join(packageDir, "scripts", "install.js")], {
      env: {
        ...process.env,
        AI_STATUSLINE_DOWNLOAD_BASE_URL: `http://127.0.0.1:${port}`,
      },
    });

    const installedBin = path.join(
      prefixDir,
      "node_modules",
      ".bin",
      process.platform === "win32" ? "ai-statusline.cmd" : "ai-statusline"
    );
    const versionOutput = execFileSync(installedBin, ["--version"], {
      encoding: "utf8",
    }).trim();
    if (!versionOutput.includes(packageJson.version)) {
      throw new Error(`Unexpected --version output: ${versionOutput}`);
    }

    console.log(`packed npm install smoke passed: ${versionOutput}`);
  } finally {
    fs.rmSync(tarballPath, { force: true });
  }
} finally {
  server.close();
  fs.rmSync(scratchDir, { force: true, recursive: true });
}
