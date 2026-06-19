#!/bin/sh
# whisper-rs-sys (0.10..0.15) vendors whisper.cpp without its `tests/` or
# `examples/` source dirs, but whisper.cpp's CMakeLists.txt references them
# with `add_subdirectory(tests)` / `add_subdirectory(examples)` unconditionally
# (gated only on `WHISPER_BUILD_TESTS` / `WHISPER_BUILD_EXAMPLES`, both ON by
# default when WHISPER_STANDALONE=ON). CMake configure fails.
#
# This wrapper injects `-DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=OFF`
# (and for good measure -DWHISPER_STANDALONE=OFF) into every cmake invocation.
# Harmless on the `cmake --build` call (unknown defines are ignored there).
# Companion: tools/macos-clang-wrap.sh + tools/macos-clangxx-wrap.sh.
# Usage: CMAKE=/.../macos-cmake-wrap.sh bun run tauri build

REAL_CMAKE="${REAL_CMAKE:-/usr/local/bin/cmake}"

# Only inject on a configure-style call (one where the first positional arg
# looks like a path, not a subcommand like `--build`, `--install`, `-E`).
case "$1" in
  --build|--install|-E|--find-package|--system-information|--help*|--version)
    exec "$REAL_CMAKE" "$@"
    ;;
esac

exec "$REAL_CMAKE" \
  -DWHISPER_BUILD_TESTS=OFF \
  -DWHISPER_BUILD_EXAMPLES=OFF \
  -DWHISPER_STANDALONE=OFF \
  "$@"
