const path = require("node:path");

const ext = process.platform === "win32" ? ".exe" : "";
module.exports = {
  binaryPath: path.join(import.meta.dirname, "bin", `claude-status-bin${ext}`),
};
