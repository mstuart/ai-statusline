#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import https from "node:https";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PACKAGE = JSON.parse(
  fs.readFileSync(new URL("../package.json", import.meta.url), "utf8")
);
const VERSION = PACKAGE.version;
const REPO = "mstuart/ai-statusline";
const PACKAGE_NAME = "ai-statusline";
const BINARY_BASENAME = "ai-statusline-bin";
const DOWNLOAD_BASE_URL = process.env.AI_STATUSLINE_DOWNLOAD_BASE_URL;
const TRAILING_SLASH_PATTERN = /\/$/;
const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));

function getPlatformTarget() {
  const platform = os.platform();
  const arch = os.arch();

  const targets = {
    "darwin-arm64": "aarch64-apple-darwin",
    "darwin-x64": "x86_64-apple-darwin",
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "win32-x64": "x86_64-pc-windows-msvc",
  };

  const key = `${platform}-${arch}`;
  const target = targets[key];

  if (!target) {
    console.error(`Unsupported platform: ${key}`);
    console.error(`Supported platforms: ${Object.keys(targets).join(", ")}`);
    process.exit(1);
  }

  return { arch, platform, target };
}

function getArchiveName(target) {
  const extension = target.includes("windows") ? "zip" : "tar.gz";
  return `${PACKAGE_NAME}-v${VERSION}-${target}.${extension}`;
}

function getDownloadUrl(target) {
  const archiveName = getArchiveName(target);

  if (DOWNLOAD_BASE_URL) {
    return `${DOWNLOAD_BASE_URL.replace(TRAILING_SLASH_PATTERN, "")}/${archiveName}`;
  }

  return `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;
}

function download(url) {
  return new Promise((resolve, reject) => {
    const handler = (response) => {
      if (
        response.statusCode >= 300 &&
        response.statusCode < 400 &&
        response.headers.location
      ) {
        const redirectUrl = new URL(response.headers.location, url).toString();
        const client = redirectUrl.startsWith("https") ? https : http;
        client.get(redirectUrl, handler).on("error", reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(
          new Error(
            `Download failed with status ${response.statusCode}: ${url}`
          )
        );
        return;
      }

      const chunks = [];
      response.on("data", (chunk) => chunks.push(chunk));
      response.on("end", () => resolve(Buffer.concat(chunks)));
      response.on("error", reject);
    };

    const client = url.startsWith("https") ? https : http;
    client.get(url, handler).on("error", reject);
  });
}

function extractTarGz(buffer, destDir) {
  const tmpFile = path.join(os.tmpdir(), `ai-statusline-${Date.now()}.tar.gz`);
  fs.writeFileSync(tmpFile, buffer);

  try {
    execFileSync("tar", ["xzf", tmpFile, "-C", destDir], { stdio: "pipe" });
  } finally {
    try {
      fs.unlinkSync(tmpFile);
    } catch {
      // The temporary archive may already have been removed.
    }
  }
}

function extractZip(buffer, destDir) {
  const tmpFile = path.join(os.tmpdir(), `ai-statusline-${Date.now()}.zip`);
  fs.writeFileSync(tmpFile, buffer);

  try {
    execFileSync("unzip", ["-o", tmpFile, "-d", destDir], { stdio: "pipe" });
  } finally {
    try {
      fs.unlinkSync(tmpFile);
    } catch {
      // The temporary archive may already have been removed.
    }
  }
}

async function install() {
  const { target, platform } = getPlatformTarget();
  const binDir = path.join(SCRIPT_DIR, "..", "bin");
  const binaryName =
    platform === "win32" ? `${BINARY_BASENAME}.exe` : BINARY_BASENAME;
  const binPath = path.join(binDir, binaryName);

  if (fs.existsSync(binPath)) {
    console.log(`ai-statusline binary already installed at ${binPath}`);
    return;
  }

  const url = getDownloadUrl(target);
  console.log(`Downloading ai-statusline v${VERSION} for ${target}...`);
  console.log(`  URL: ${url}`);

  try {
    const data = await download(url);
    console.log(`  Downloaded ${(data.length / 1024 / 1024).toFixed(1)} MB`);

    fs.mkdirSync(binDir, { recursive: true });

    if (target.includes("windows")) {
      extractZip(data, binDir);
    } else {
      extractTarGz(data, binDir);
    }

    if (platform !== "win32") {
      fs.chmodSync(binPath, 0o755);
    }

    console.log(`  Installed to ${binPath}`);
  } catch (error) {
    console.warn(`\nFailed to download pre-built binary: ${error.message}`);
    console.warn("\nYou can build from source instead:");
    console.warn("  cargo install --path .");
    console.warn("\nOr download manually from:");
    console.warn(`  https://github.com/${REPO}/releases`);

    const stub =
      platform === "win32"
        ? "@echo off\necho ai-statusline binary not installed. Run: cargo install --path . in the ai-statusline repo\nexit /b 1\n"
        : '#!/bin/sh\necho "ai-statusline binary not installed. Run: cargo install --path . in the ai-statusline repo"\nexit 1\n';

    fs.mkdirSync(binDir, { recursive: true });
    fs.writeFileSync(binPath, stub);
    if (platform !== "win32") {
      fs.chmodSync(binPath, 0o755);
    }
  }
}

install().catch((error) => {
  console.error("Installation failed:", error.message);
  process.exit(1);
});
