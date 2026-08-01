#!/usr/bin/env bash
# Materialize crates.io sources + apply patches/*.patch into patched/.
# Must run before `cargo` resolves [patch.crates-io] (see package.json scripts).
#
# Does NOT call `cargo fetch` (that would recurse into this [patch] path).
# Sources come from the local registry cache, or crates.io tarballs as fallback.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="26.5.12"
OUT="$ROOT/patched"
PATCH_DIR="$ROOT/patches"
REGISTRY_SRC="${CARGO_HOME:-$HOME/.cargo}/registry/src"

find_registry_crate() {
  local crate="$1"
  find "$REGISTRY_SRC" -maxdepth 2 -type d -name "${crate}-${VERSION}" 2>/dev/null | head -n1 || true
}

download_crate() {
  local crate="$1"
  local dest="$2"
  local tmp
  tmp="$(mktemp -d)"
  echo "downloading ${crate}-${VERSION} from crates.io…"
  curl -fsSL "https://static.crates.io/crates/${crate}/${crate}-${VERSION}.crate" \
    | tar -xz -C "$tmp"
  rm -rf "$dest"
  mkdir -p "$OUT"
  mv "$tmp/${crate}-${VERSION}" "$dest"
  rm -rf "$tmp"
}

apply_one() {
  local crate="$1"
  local dest="$OUT/${crate}-${VERSION}"
  local patch_file="$PATCH_DIR/${crate}+${VERSION}.patch"
  local stamp="$dest/.rslauncher-patch-stamp"
  local patch_hash

  if [[ ! -f "$patch_file" ]]; then
    echo "error: missing $patch_file" >&2
    exit 1
  fi

  patch_hash="$(cksum <"$patch_file" | awk '{print $1" "$2}')"

  if [[ -f "$stamp" && -f "$dest/Cargo.toml" ]] && [[ "$(cat "$stamp")" == "$patch_hash" ]]; then
    echo "ok: ${crate}-${VERSION} already patched"
    return 0
  fi

  local src
  src="$(find_registry_crate "$crate")"
  if [[ -n "$src" ]]; then
    rm -rf "$dest"
    mkdir -p "$OUT"
    cp -a "$src" "$dest"
    rm -f "$dest/.cargo-ok" "$dest/.cargo_vcs_info.json"
  else
    download_crate "$crate" "$dest"
  fi

  echo "patching ${crate}-${VERSION}…"
  (cd "$dest" && patch -p1 --forward --quiet <"$patch_file")
  printf '%s\n' "$patch_hash" >"$stamp"
  echo "ok: wrote $dest"
}

mkdir -p "$OUT"
apply_one lighty-modsloader
apply_one lighty-launch
apply_one lighty-java
