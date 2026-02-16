#!/usr/bin/env sh
set -e

REPO="mstuart/claudeline"
INSTALL_DIR="${CLAUDELINE_INSTALL_DIR:-$HOME/.local/bin}"

main() {
  detect_platform
  detect_arch

  binary="claudeline-${PLATFORM}-${ARCH}"
  url="https://github.com/${REPO}/releases/latest/download/${binary}"

  echo "Downloading claudeline for ${PLATFORM}-${ARCH}..."
  echo "  ${url}"

  tmpdir=$(mktemp -d)
  trap "rm -rf ${tmpdir}" EXIT

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "${tmpdir}/claudeline" "${url}"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "${tmpdir}/claudeline" "${url}"
  else
    echo "Error: curl or wget is required to download claudeline."
    exit 1
  fi

  chmod +x "${tmpdir}/claudeline"

  if [ -w "${INSTALL_DIR}" ] || mkdir -p "${INSTALL_DIR}" 2>/dev/null; then
    cp "${tmpdir}/claudeline" "${INSTALL_DIR}/claudeline"
  else
    echo "Installing to ${INSTALL_DIR} requires elevated permissions."
    sudo mkdir -p "${INSTALL_DIR}"
    sudo cp "${tmpdir}/claudeline" "${INSTALL_DIR}/claudeline"
  fi

  echo ""
  echo "claudeline installed to ${INSTALL_DIR}/claudeline"
  echo ""

  if ! echo "$PATH" | tr ':' '\n' | grep -q "^${INSTALL_DIR}$"; then
    echo "Add ${INSTALL_DIR} to your PATH:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
  fi

  echo "To use with Claude Code, add to ~/.claude/settings.json:"
  echo '  {'
  echo '    "statusLine": {'
  echo '      "type": "command",'
  echo '      "command": "claudeline"'
  echo '    }'
  echo '  }'
}

detect_platform() {
  case "$(uname -s)" in
    Darwin)  PLATFORM="darwin" ;;
    Linux)   PLATFORM="linux" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    *)
      echo "Error: Unsupported platform: $(uname -s)"
      exit 1
      ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)  ARCH="x64" ;;
    aarch64|arm64)  ARCH="arm64" ;;
    *)
      echo "Error: Unsupported architecture: $(uname -m)"
      exit 1
      ;;
  esac
}

main "$@"
