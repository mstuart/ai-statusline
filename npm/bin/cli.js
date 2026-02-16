#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');

const ext = process.platform === 'win32' ? '.exe' : '';
const bin = path.join(__dirname, `claudeline-bin${ext}`);

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: false,
});

if (result.error) {
  if (result.error.code === 'ENOENT') {
    console.error('claudeline binary not found. Try reinstalling: npm install -g claudeline');
  } else {
    console.error(result.error.message);
  }
  process.exit(1);
}

process.exit(result.status ?? 1);
