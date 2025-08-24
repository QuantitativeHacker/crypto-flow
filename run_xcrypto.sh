#!/usr/bin/env bash
set -euo pipefail

# One-click setup and run for xcrypto with minimal interference to host.
# Default: use Docker to isolate toolchains and system libs.
# Fallback: use local environment (venv + cargo) without installing system packages.
# Usage:
#   ./run_xcrypto.sh [spot|usdt]
# Examples:
#   ./run_xcrypto.sh          # default spot
#   ./run_xcrypto.sh usdt     # run usdt futures

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/xcrypto"
WORKSPACE_DIR="$PROJECT_ROOT"
MODE="${1:-spot}"
if [[ "$MODE" != "spot" && "$MODE" != "usdt" ]]; then
  echo "[ERROR] Unknown mode: $MODE (expected 'spot' or 'usdt')" >&2
  exit 1
fi

CONFIG_JSON="$PROJECT_ROOT/${MODE}.json"
if [[ ! -f "$CONFIG_JSON" ]]; then
  echo "[ERROR] Missing config file: $CONFIG_JSON" >&2
  exit 1
fi

PEM_FILE="$PROJECT_ROOT/private_key.pem"
if [[ ! -f "$PEM_FILE" ]]; then
  echo "[ERROR] Missing private key: $PEM_FILE" >&2
  exit 1
fi

CONFIG_BASENAME="$(basename "$CONFIG_JSON")"

run_in_docker() {
  echo "[INFO] Using Docker isolation (recommended, minimal host changes)"
  # Map port 8111 for WS server; mount workspace and build/run inside container.
  # Use named volumes for caching to avoid recompilation
  docker run --rm -it \
    -p 8111:8111 \
    -v "$WORKSPACE_DIR":/workspace \
    -v "$SCRIPT_DIR":/strategies \
    -v xcrypto-cargo-cache:/usr/local/cargo/registry \
    -v xcrypto-target-cache:/workspace/target \
    -w /workspace \
    rust:1.80-bookworm \
    bash -c "set -e; \
      export DEBIAN_FRONTEND=noninteractive; \
      apt-get update -y; \
      apt-get install -y --no-install-recommends \
        ca-certificates build-essential python3 python3-venv python3-pip pkg-config libssl-dev libsqlite3-dev; \
      update-ca-certificates; \
      CARGO_BIN=/usr/local/cargo/bin/cargo; \
      if [ ! -x \$CARGO_BIN ]; then CARGO_BIN=\"cargo\"; fi; \
      echo \"[INFO] Using cargo at: \$(command -v \$CARGO_BIN || echo \$CARGO_BIN)\"; \
      \$CARGO_BIN --version || true; \
      # Build Rust binary
      cd /workspace/binance/$MODE && \$CARGO_BIN build -r; \
      # Build Python strategy package (inside project-only venv)
      cd /workspace/pyalgo && python3 -m venv .venv && source .venv/bin/activate && \
        python -m pip install --upgrade pip 'maturin>=1.5,<2.0' && \
        maturin develop -r; \
      # Run server from project root so config & pem are discoverable
      cd /workspace && \
      echo '[INFO] Launching $MODE server on ws://localhost:8111' && \
      /workspace/target/release/$MODE -c=$CONFIG_BASENAME -l=info"
}

run_locally() {
  echo "[INFO] Docker not found. Falling back to local run (no system installs)."
  # Sanity checks without modifying host
  if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERROR] 'cargo' not found. Please install Rust (rustup) and retry." >&2
    exit 1
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    echo "[ERROR] 'python3' not found. Please install Python 3.10+ and retry." >&2
    exit 1
  fi
  # Check system libs availability but do not install automatically
  if ! command -v pkg-config >/dev/null 2>&1; then
    echo "[WARN] 'pkg-config' not found. OpenSSL/SQLite detection may fail at build time." >&2
  else
    if ! pkg-config --exists openssl; then
      echo "[WARN] OpenSSL dev libraries not detected by pkg-config. Build may fail."
    fi
    if ! pkg-config --exists sqlite3; then
      echo "[WARN] SQLite3 dev libraries not detected by pkg-config. Build may fail."
    fi
  fi

  # Build Rust binary (release)
  pushd "$WORKSPACE_DIR/binance/$MODE" >/dev/null
  cargo build -r
  popd >/dev/null

  # Build Python strategy package in project-local venv
  pushd "$WORKSPACE_DIR/pyalgo" >/dev/null
  python3 -m venv .venv
  source .venv/bin/activate
  python -m pip install --upgrade pip 'maturin>=1.5,<2.0'
  maturin develop -r
  deactivate || true
  popd >/dev/null

  # Run from project root so that private_key.pem resolves (pem path is relative in JSON)
  echo "[INFO] Launching $MODE server on ws://localhost:8111"
  "$WORKSPACE_DIR/target/release/$MODE" -c="$CONFIG_JSON" -l=info
}

if command -v docker >/dev/null 2>&1; then
  run_in_docker
else
  run_locally
fi