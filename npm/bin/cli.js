#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');

const ext = process.platform === 'win32' ? '.exe' : '';
const bin = path.join(__dirname, `claude-status-bin${ext}`);

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: false,
});

if (result.error) {
  if (result.error.code === 'ENOENT') {
    console.error('claude-status binary not found. Try reinstalling: npm install -g claude-status');
  } else {
    console.error(result.error.message);
  }
  process.exit(1);
}

process.exit(result.status ?? 1);
