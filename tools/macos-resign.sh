#!/usr/bin/env bash
# Re-sign Dictatr.app ad-hoc WITHOUT hardened runtime.
#
# macOS 26 beta blocks TCC permission prompts (Mikrofon, Bedienungshilfen) for
# ad-hoc-signed apps when hardened runtime is enabled — the system silently
# resolves `AVCaptureDevice.requestAccess` as "denied" without showing a
# dialog. Stripping the runtime flag lets TCC prompt the user properly.
#
# Tauri's bundler always enables hardened runtime for macOS builds. Run this
# script after `bun run tauri build` to fix the produced .app (and the copy
# you installed under /Applications, if any).
#
# Usage:
#   ./tools/macos-resign.sh                    # resigns target/release/bundle/macos/Dictatr.app
#   ./tools/macos-resign.sh /Applications/Dictatr.app  # any explicit bundle
set -euo pipefail

APP="${1:-src-tauri/target/release/bundle/macos/Dictatr.app}"

if [[ ! -d "$APP" ]]; then
    echo "error: $APP not found" >&2
    exit 1
fi

codesign --force --deep --sign - "$APP"
echo "re-signed $APP without hardened runtime"
codesign -dv "$APP" 2>&1 | grep -E "flags|Signature"
