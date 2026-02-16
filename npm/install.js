const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const REPO = 'mstuart/claudeline';
const BIN_DIR = path.join(__dirname, 'bin');
const BINARY_NAME = process.platform === 'win32' ? 'claudeline-bin.exe' : 'claudeline-bin';
const BIN_PATH = path.join(BIN_DIR, BINARY_NAME);

function getPlatform() {
  const platform = process.platform;
  switch (platform) {
    case 'darwin': return 'darwin';
    case 'linux': return 'linux';
    case 'win32': return 'windows';
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
}

function getArch() {
  const arch = process.arch;
  switch (arch) {
    case 'x64': return 'x64';
    case 'arm64': return 'arm64';
    default:
      throw new Error(`Unsupported architecture: ${arch}`);
  }
}

function getBinaryName(platform, arch) {
  const ext = platform === 'windows' ? '.exe' : '';
  return `claudeline-${platform}-${arch}${ext}`;
}

function fetch(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith('https') ? https : http;
    client.get(url, { headers: { 'User-Agent': 'claudeline-npm-install' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return fetch(res.headers.location).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode}: ${url}`));
      }
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => resolve(Buffer.concat(chunks)));
      res.on('error', reject);
    }).on('error', reject);
  });
}

async function install() {
  const platform = getPlatform();
  const arch = getArch();
  const binaryName = getBinaryName(platform, arch);
  const url = `https://github.com/${REPO}/releases/latest/download/${binaryName}`;

  console.log(`Downloading claudeline binary for ${platform}-${arch}...`);
  console.log(`  ${url}`);

  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  const data = await fetch(url);
  fs.writeFileSync(BIN_PATH, data);

  if (process.platform !== 'win32') {
    fs.chmodSync(BIN_PATH, 0o755);
  }

  console.log(`claudeline binary installed to ${BIN_PATH}`);
}

install().catch((err) => {
  console.error('Failed to download claudeline binary.');
  console.error(err.message);
  console.error('');
  console.error('You can install manually:');
  console.error(`  1. Download the binary from https://github.com/${REPO}/releases/latest`);
  console.error(`  2. Save it to ${BIN_PATH}`);
  if (process.platform !== 'win32') {
    console.error(`  3. Run: chmod +x ${BIN_PATH}`);
  }
  process.exit(1);
});
