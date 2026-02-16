const path = require('path');

const ext = process.platform === 'win32' ? '.exe' : '';
module.exports = {
  binaryPath: path.join(__dirname, 'bin', `claudeline-bin${ext}`)
};
