#!/bin/sh
# See tools/macos-clang-wrap.sh — same idea for clang++.

REAL_CXX="${REAL_CXX:-/usr/bin/clang++}"

args=""
for a in "$@"; do
  case "$a" in
    -mcpu=native|-mcpu=native+*|-march=native)
      args="$args -mcpu=apple-m1"
      ;;
    -mavx|-mavx2|-mfma|-mf16c|-msse*|-mssse3|-mavx512*)
      ;;
    *)
      args="$args $a"
      ;;
  esac
done

exec "$REAL_CXX" $args
