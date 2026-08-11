#!/usr/bin/env sh
set -e

REPO="mstuart/ai-statusline"
INSTALL_DIR="${AI_STATUSLINE_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${AI_STATUSLINE_VERSION:-latest}"
DOWNLOAD_BASE_URL="${AI_STATUSLINE_DOWNLOAD_BASE_URL:-}"

main() {
  detect_target
  release_tag="${VERSION}"
  if [ "${VERSION}" != "latest" ]; then
    case "${VERSION}" in
      v*) ;;
      *) release_tag="v${VERSION}" ;;
    esac
  fi

  archive="ai-statusline-${release_tag}-${TARGET}.${EXTENSION}"
  if [ "${VERSION}" = "latest" ]; then
    release_tag=$(latest_tag)
    archive="ai-statusline-${release_tag}-${TARGET}.${EXTENSION}"
  fi

  if [ -n "${DOWNLOAD_BASE_URL}" ]; then
    url="${DOWNLOAD_BASE_URL%/}/${archive}"
  else
    url="https://github.com/${REPO}/releases/download/${release_tag}/${archive}"
    if [ "${VERSION}" = "latest" ]; then
      url="https://github.com/${REPO}/releases/latest/download/${archive}"
    fi
  fi

  echo "Downloading ai-statusline for ${TARGET}..."
  echo "  ${url}"

  tmpdir=$(mktemp -d)
  trap 'rm -rf "${tmpdir}"' EXIT

  download "${url}" "${tmpdir}/${archive}"
  extract "${tmpdir}/${archive}" "${tmpdir}"
  chmod +x "${tmpdir}/${BINARY_NAME}"

  if [ -w "${INSTALL_DIR}" ] || mkdir -p "${INSTALL_DIR}" 2>/dev/null; then
    cp "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/ai-statusline"
  else
    echo "Installing to ${INSTALL_DIR} requires elevated permissions."
    sudo mkdir -p "${INSTALL_DIR}"
    sudo cp "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/ai-statusline"
  fi

  echo ""
  echo "ai-statusline installed to ${INSTALL_DIR}/ai-statusline"
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
  echo '      "command": "ai-statusline"'
  echo '    }'
  echo '  }'
}

latest_tag() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" | sed 's#.*/##'
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1
  else
    echo "Error: curl or wget is required to resolve the latest ai-statusline release." >&2
    exit 1
  fi
}

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "${output}" "${url}"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "${output}" "${url}"
  else
    echo "Error: curl or wget is required to download ai-statusline."
    exit 1
  fi
}

extract() {
  archive_path="$1"
  output_dir="$2"
  if [ "${EXTENSION}" = "zip" ]; then
    unzip -o "${archive_path}" -d "${output_dir}" >/dev/null
  else
    tar xzf "${archive_path}" -C "${output_dir}"
  fi
}

detect_target() {
  case "$(uname -s)" in
    Darwin) PLATFORM="apple-darwin" ;;
    Linux) PLATFORM="unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="pc-windows-msvc" ;;
    *)
      echo "Error: Unsupported platform: $(uname -s)"
      exit 1
      ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
      echo "Error: Unsupported architecture: $(uname -m)"
      exit 1
      ;;
  esac

  TARGET="${ARCH}-${PLATFORM}"
  BINARY_NAME="ai-statusline-bin"
  EXTENSION="tar.gz"
  if [ "${PLATFORM}" = "pc-windows-msvc" ]; then
    BINARY_NAME="ai-statusline-bin.exe"
    EXTENSION="zip"
  fi
}

main "$@"
