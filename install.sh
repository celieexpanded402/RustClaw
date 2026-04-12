#!/bin/sh
set -e

REPO="Adaimade/RustClaw"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.rustclaw"

echo "RustClaw Installer"
echo "====================="

cleanup() { rm -rf "$TMPDIR" 2>/dev/null; }
trap cleanup EXIT

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64) TARGET="rustclaw-aarch64-apple-darwin" ;;
            x86_64) TARGET="rustclaw-x86_64-apple-darwin" ;;
            *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Linux)
        case "$ARCH" in
            x86_64) TARGET="rustclaw-x86_64-linux" ;;
            aarch64) TARGET="rustclaw-aarch64-linux" ;;
            *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "❌ Unsupported OS: $OS (use install.ps1 for Windows)"
        exit 1
        ;;
esac

echo "Platform: $OS $ARCH -> $TARGET"

# Get latest release URL
RELEASE_JSON=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest")
LATEST=$(echo "$RELEASE_JSON" | grep "browser_download_url" | grep "$TARGET.tar.gz" | head -1 | cut -d '"' -f 4)

if [ -z "$LATEST" ]; then
    echo "ERROR: Could not find release for $TARGET"
    echo "  Check: https://github.com/$REPO/releases"
    echo "  If this is a fresh install, the project may not have published releases yet."
    echo "  Build from source instead: git clone https://github.com/$REPO && cd RustClaw && cargo build --release"
    exit 1
fi

echo "Downloading: $LATEST"

# Download and verify
TMPDIR=$(mktemp -d)
curl -sL "$LATEST" -o "$TMPDIR/rustclaw.tar.gz"

# SHA256 checksum verification
CHECKSUM_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*sha256" | head -1 | cut -d '"' -f 4)
if [ -n "$CHECKSUM_URL" ]; then
    curl -sL "$CHECKSUM_URL" -o "$TMPDIR/checksums.txt"
    EXPECTED=$(grep "$TARGET.tar.gz" "$TMPDIR/checksums.txt" | awk '{print $1}')
    if [ -n "$EXPECTED" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL=$(sha256sum "$TMPDIR/rustclaw.tar.gz" | awk '{print $1}')
        else
            ACTUAL=$(shasum -a 256 "$TMPDIR/rustclaw.tar.gz" | awk '{print $1}')
        fi
        if [ "$ACTUAL" != "$EXPECTED" ]; then
            echo "ERROR: Checksum verification failed!"
            echo "  Expected: $EXPECTED"
            echo "  Got:      $ACTUAL"
            exit 1
        fi
        echo "[OK] Checksum verified"
    fi
fi

tar xzf "$TMPDIR/rustclaw.tar.gz" -C "$TMPDIR"

# Find and install binary
BINARY=$(find "$TMPDIR" -name "rustclaw" -type f | head -1)
if [ -z "$BINARY" ]; then
    echo "ERROR: rustclaw binary not found in archive"
    exit 1
fi

mkdir -p "$INSTALL_DIR"
mv "$BINARY" "$INSTALL_DIR/rustclaw"
chmod +x "$INSTALL_DIR/rustclaw"

echo "Installed: $INSTALL_DIR/rustclaw"

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "WARNING: $INSTALL_DIR is not in your PATH. Add it:"
        echo ""
        SHELL_NAME=$(basename "$SHELL" 2>/dev/null || echo "bash")
        case "$SHELL_NAME" in
            zsh)  echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && source ~/.zshrc" ;;
            *)    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc" ;;
        esac
        echo ""
        ;;
esac

# Create default config
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_DIR/config.toml" << 'TOML'
[gateway]
port = 18789
bind = "127.0.0.1"
token = ""

[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen3-coder:30b"
system_prompt = "You are a helpful assistant."

[channels.telegram]
enabled = false
bot_token = ""

[channels.discord]
enabled = false
bot_token = ""
TOML
    echo "Config created: $CONFIG_DIR/config.toml"
    echo "  Default: Ollama local mode (edit config to use Anthropic/OpenAI API)"
else
    echo "Config exists: $CONFIG_DIR/config.toml (not overwritten)"
fi

# Check Ollama
echo ""
if command -v ollama >/dev/null 2>&1; then
    echo "[OK] Ollama detected"
    if curl -s --max-time 2 http://localhost:11434/api/tags >/dev/null 2>&1; then
        echo "[OK] Ollama is running"
        MODELS=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | head -5)
        if [ -n "$MODELS" ]; then
            echo "  Installed models:"
            echo "$MODELS" | sed 's/"name":"/ - /;s/"//'
        fi
    else
        echo "[!] Ollama installed but not running. Start it: ollama serve"
    fi
else
    echo "[i] Ollama not found. For local LLM support:"
    echo "    https://ollama.com/download"
    echo ""
    echo "    Or use a cloud API - edit $CONFIG_DIR/config.toml:"
    echo "    provider = \"anthropic\""
    echo "    api_key = \"sk-ant-...\""
fi

echo ""
echo "Done! Try:"
echo "  rustclaw agent \"Hello, what can you do?\""
echo "  rustclaw gateway   # Start server with Telegram/Discord"
echo ""
