#!/usr/bin/env bash
set -e

# cocode installer
# usage: curl -fsSL https://raw.githubusercontent.com/YOUR_USER/cocode/main/install.sh | bash

REPO="https://github.com/YOUR_USER/cocode"
BIN="cocode"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# ── helpers ──────────────────────────────────────────────────────────────────

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
info() { printf '  \033[34m•\033[0m %s\n' "$*"; }
ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
err()  { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

# ── checks ───────────────────────────────────────────────────────────────────

check_dep() {
    command -v "$1" &>/dev/null || err "$1 is required but not found. install it and try again."
}

# ── main ─────────────────────────────────────────────────────────────────────

bold ""
bold "  cocode installer"
echo ""

# prefer a pre-built release binary if gh releases are available,
# otherwise fall back to building from source.

# check if we're running piped from curl (no local repo)
if [[ ! -f "Cargo.toml" ]]; then
    # remote install — clone and build
    info "no local repo found, cloning from github..."
    check_dep git
    check_dep cargo

    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    git clone --depth 1 "$REPO" "$TMPDIR/cocode" 2>&1 | grep -v "^$" || true
    cd "$TMPDIR/cocode"
else
    # local install — already in the repo
    info "building from local source..."
    check_dep cargo
fi

info "compiling (release)..."
cargo build --release --quiet

mkdir -p "$INSTALL_DIR"
cp target/release/$BIN "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

ok "installed → $INSTALL_DIR/$BIN"

# ── PATH hint ────────────────────────────────────────────────────────────────

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo ""
    bold "  add this to your shell rc:"
    echo ""
    printf '    export PATH="%s:$PATH"\n' "$INSTALL_DIR"
    echo ""
    info "then restart your shell or run: source ~/.bashrc  (or ~/.zshrc)"
else
    echo ""
    ok "cocode is ready — run: cocode"
fi

echo ""
