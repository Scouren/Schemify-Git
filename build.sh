#!/usr/bin/env bash
# Builds every plugin of this repository for wasm32-unknown-unknown into
# crates/<name>/<name>.wasm (committed: a Schemify package is a prebuilt module, config spec
# §5; Schemify pins each by commit and sha256). Fails if a module reaches 200 KB.
#
# Reproducible: CI rebuilds the modules and fails when a committed one differs. The bytes
# depend only on the sources, Cargo.lock and the toolchain: build in `nix develop` (its
# rust-overlay input pins rustc and rust-lld). Host paths are remapped out of the modules.
#
# Without a rust-lld on the toolchain it links with wasm-ld, re-running itself under
# `nix shell nixpkgs#lld` when wasm-ld is not on PATH either.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
host="$(rustc -vV | sed -n 's/^host: //p')"
pinned="$(sed -n 's/^rust-version = "\(.*\)"/\1/p' "$here/Cargo.toml")"
if ! rustc -V | grep -q "^rustc $pinned\(\.0\)\? "; then
  echo "warning: $(rustc -V) is not the pinned rustc $pinned (nix develop): the modules will" \
    "not match the committed ones byte for byte" >&2
fi
# Unit separator between flags: paths may contain spaces.
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$here=/Schemify-Git"$'\x1f'"--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo"

if [[ ! -x "$(rustc --print sysroot)/lib/rustlib/$host/bin/rust-lld" ]] \
   && [[ -z "${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER:-}" ]]; then
  if command -v wasm-ld >/dev/null; then
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld
  else
    exec nix shell nixpkgs#lld -c env CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld "$0" "$@"
  fi
fi

cargo build --manifest-path "$here/Cargo.toml" --release --target wasm32-unknown-unknown --lib

target="${CARGO_TARGET_DIR:-$here/target}"
for name in git hg jj svn p4; do
  wasm="$target/wasm32-unknown-unknown/release/$name.wasm"
  size=$(wc -c <"$wasm")
  if (( size >= 200 * 1024 )); then
    echo "error: $name.wasm is $size bytes (limit 200 KB)" >&2
    exit 1
  fi
  cp "$wasm" "$here/crates/$name/$name.wasm"
  echo "$name.wasm $size bytes"
done
