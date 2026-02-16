const path = require('path');

const ext = process.platform === 'win32' ? '.exe' : '';
module.exports = {
  binaryPath: path.join(__dirname, 'bin', `claude-status-bin${ext}`)
};
