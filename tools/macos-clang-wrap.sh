#!/bin/sh
# Apple clang on macOS 26 beta rejects a handful of compiler flags that
# whisper.cpp's CMake build passes unconditionally:
#   * `-mcpu=native` / `-mcpu=native+feature+…` — Apple clang wants
#     `-mcpu=apple-m1` or explicit `-Xclang -target-feature +…`
#   * `-mavx`, `-mavx2`, `-mfma`, `-mf16c`, `-mssse3`, etc. — x86-only,
#     invalid on arm64 targets
#
# This wrapper rewrites / drops those flags so the build proceeds on
# Apple Silicon. Companion: tools/macos-clangxx-wrap.sh.
# Usage: CC=.../macos-clang-wrap.sh CXX=.../macos-clangxx-wrap.sh

REAL_CC="${REAL_CC:-/usr/bin/clang}"

args=""
for a in "$@"; do
  case "$a" in
    -mcpu=native|-mcpu=native+*|-march=native)
      args="$args -mcpu=apple-m1"
      ;;
    -mavx|-mavx2|-mfma|-mf16c|-msse*|-mssse3|-mavx512*)
      # drop silently on arm64 — these are x86-only
      ;;
    *)
      args="$args $a"
      ;;
  esac
done

exec "$REAL_CC" $args
