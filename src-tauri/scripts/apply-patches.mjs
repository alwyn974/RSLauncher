#!/usr/bin/env node
// Materialize crates.io sources + apply patches/*.patch into patched/.
// Must run before `cargo` resolves [patch.crates-io] (see package.json scripts).
//
// Does NOT call `cargo fetch` (that would recurse into this [patch] path).
// Sources come from the local registry cache, or crates.io tarballs as fallback.

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const VERSION = "26.5.12";
const OUT = join(ROOT, "patched");
const PATCH_DIR = join(ROOT, "patches");
const REGISTRY_SRC = join(
  process.env.CARGO_HOME ?? join(homedir(), ".cargo"),
  "registry",
  "src",
);

function findRegistryCrate(crate) {
  if (!existsSync(REGISTRY_SRC)) {
    return null;
  }
  const target = `${crate}-${VERSION}`;
  for (const entry of readdirSync(REGISTRY_SRC, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const candidate = join(REGISTRY_SRC, entry.name, target);
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

async function downloadCrate(crate, dest) {
  const tmp = mkdtempSync(join(tmpdir(), "rslauncher-crate-"));
  const url = `https://static.crates.io/crates/${crate}/${crate}-${VERSION}.crate`;
  const cratePath = join(tmp, `${crate}-${VERSION}.crate`);

  console.log(`downloading ${crate}-${VERSION} from crates.io…`);
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`failed to download ${url}: ${res.status} ${res.statusText}`);
  }
  writeFileSync(cratePath, Buffer.from(await res.arrayBuffer()));

  execFileSync("tar", ["-xzf", cratePath, "-C", tmp], { stdio: "inherit" });

  rmSync(dest, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  renameSync(join(tmp, `${crate}-${VERSION}`), dest);
  rmSync(tmp, { recursive: true, force: true });
}

async function applyOne(crate) {
  const dest = join(OUT, `${crate}-${VERSION}`);
  const patchFile = join(PATCH_DIR, `${crate}+${VERSION}.patch`);
  const stamp = join(dest, ".rslauncher-patch-stamp");

  if (!existsSync(patchFile)) {
    console.error(`error: missing ${patchFile}`);
    process.exit(1);
  }

  const patchHash = createHash("sha256").update(readFileSync(patchFile)).digest("hex");

  if (
    existsSync(stamp) &&
    existsSync(join(dest, "Cargo.toml")) &&
    readFileSync(stamp, "utf8").trim() === patchHash
  ) {
    console.log(`ok: ${crate}-${VERSION} already patched`);
    return;
  }

  const src = findRegistryCrate(crate);
  if (src) {
    rmSync(dest, { recursive: true, force: true });
    mkdirSync(OUT, { recursive: true });
    cpSync(src, dest, { recursive: true });
    for (const name of [".cargo-ok", ".cargo_vcs_info.json"]) {
      const p = join(dest, name);
      if (existsSync(p)) unlinkSync(p);
    }
  } else {
    await downloadCrate(crate, dest);
  }

  console.log(`patching ${crate}-${VERSION}…`);
  execFileSync(
    "git",
    ["apply", "-p1", "--whitespace=nowarn", patchFile],
    { cwd: dest, stdio: "inherit" },
  );
  writeFileSync(stamp, `${patchHash}\n`);
  console.log(`ok: wrote ${dest}`);
}

mkdirSync(OUT, { recursive: true });
await applyOne("lighty-modsloader");
await applyOne("lighty-launch");
await applyOne("lighty-java");
