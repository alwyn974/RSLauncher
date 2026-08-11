#!/usr/bin/env node
/**
 * Import a CurseForge custom-pack manifest.json into a RSLauncher pack TOML.
 *
 * Keeps a pristine copy at modpack/manifests/<id>/manifest.json for CF re-export.
 * Writes/updates [[required_curseforge]] between BEGIN/END markers; extras outside
 * that block are preserved on re-import.
 *
 * Usage:
 *   node src-tauri/scripts/import-curseforge-manifest.mjs \
 *     --id mypack \
 *     --manifest ./export/manifest.json \
 *     --server-name "Eh Zebi" \
 *     --server-address "mc.example.com"
 */

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const MODPACK_DIR = join(ROOT, "modpack");
const PACKS_DIR = join(MODPACK_DIR, "packs");
const MANIFESTS_DIR = join(MODPACK_DIR, "manifests");
const INDEX_PATH = join(MODPACK_DIR, "modpacks.toml");
const CF_API = "https://api.curseforge.com/v1";

const BEGIN = "# --- BEGIN CURSEFORGE MANIFEST (do not edit; re-run import script) ---";
const END = "# --- END CURSEFORGE MANIFEST ---";
const EXTRAS_HINT =
  "# Manual extras below this line are preserved on re-import:";

function usage(exitCode = 1) {
  console.error(`Usage:
  node src-tauri/scripts/import-curseforge-manifest.mjs \\
    --id <pack-id> \\
    --manifest <path-to-manifest.json> \\
    [--server-name <name>] \\
    [--server-address <host>] \\
    [--instance-name <name>] \\
    [--fallback-ram-gb <n>] \\
    [--min-ram-gb <n>]

Reads CURSEFORGE_API_KEY env or src-tauri/curseforge_api_key for mod names.`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const out = {
    id: null,
    manifest: null,
    serverName: "Server",
    serverAddress: "127.0.0.1",
    instanceName: null,
    fallbackRamGb: 8,
    minRamGb: 2,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => {
      const v = argv[++i];
      if (v == null || v.startsWith("--")) {
        throw new Error(`Missing value for ${a}`);
      }
      return v;
    };
    switch (a) {
      case "--id":
        out.id = next();
        break;
      case "--manifest":
        out.manifest = next();
        break;
      case "--server-name":
        out.serverName = next();
        break;
      case "--server-address":
        out.serverAddress = next();
        break;
      case "--instance-name":
        out.instanceName = next();
        break;
      case "--fallback-ram-gb":
        out.fallbackRamGb = Number(next());
        break;
      case "--min-ram-gb":
        out.minRamGb = Number(next());
        break;
      case "-h":
      case "--help":
        usage(0);
        break;
      default:
        throw new Error(`Unknown argument: ${a}`);
    }
  }
  if (!out.id || !out.manifest) {
    usage(1);
  }
  if (!/^[a-z0-9][a-z0-9_-]*$/i.test(out.id)) {
    throw new Error(`Invalid --id ${out.id} (use letters, digits, _ or -)`);
  }
  return out;
}

function readApiKey() {
  if (process.env.CURSEFORGE_API_KEY?.trim()) {
    return process.env.CURSEFORGE_API_KEY.trim();
  }
  const path = join(ROOT, "curseforge_api_key");
  if (existsSync(path)) {
    return readFileSync(path, "utf8").trim();
  }
  return null;
}

function parseLoaderId(id) {
  const idx = id.indexOf("-");
  if (idx <= 0) {
    throw new Error(`Invalid modLoader id: ${id}`);
  }
  const loader = id.slice(0, idx).toLowerCase();
  const version = id.slice(idx + 1);
  const allowed = new Set(["fabric", "forge", "neoforge", "quilt"]);
  if (!allowed.has(loader)) {
    throw new Error(`Unsupported loader "${loader}" in modLoader id ${id}`);
  }
  if (!version) {
    throw new Error(`Missing loader version in modLoader id ${id}`);
  }
  return { loader, version };
}

function escapeTomlBasic(s) {
  return JSON.stringify(s);
}

function slugifyComment(name) {
  return String(name)
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 120);
}

async function fetchModNames(apiKey, projectIds) {
  const names = new Map();
  if (projectIds.length === 0) {
    return names;
  }
  // CurseForge batch endpoint accepts up to ~1000 ids; chunk to be safe.
  const chunkSize = 100;
  for (let i = 0; i < projectIds.length; i += chunkSize) {
    const chunk = projectIds.slice(i, i + chunkSize);
    const res = await fetch(`${CF_API}/mods`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "x-api-key": apiKey,
      },
      body: JSON.stringify({ modIds: chunk }),
    });
    if (!res.ok) {
      const body = await res.text().catch(() => "");
      throw new Error(
        `CurseForge /mods failed (${res.status}): ${body.slice(0, 200)}`,
      );
    }
    const json = await res.json();
    for (const mod of json.data ?? []) {
      if (mod?.id != null && mod?.name) {
        names.set(mod.id, mod.name);
      }
    }
  }
  return names;
}

function buildManifestBlock(files, names) {
  const lines = [BEGIN];
  for (const f of files) {
    const name = names.get(f.projectID) ?? `project ${f.projectID}`;
    lines.push("[[required_curseforge]]");
    lines.push(`project_id = ${f.projectID}`);
    lines.push(`file_id = ${f.fileID}`);
    lines.push(`# ${slugifyComment(name)}`);
    lines.push("");
  }
  if (files.length === 0) {
    lines.push("# (manifest has no required files)");
    lines.push("");
  }
  lines.push(END);
  return lines.join("\n");
}

function buildSkeletonToml(opts, manifest, block) {
  const instance =
    opts.instanceName?.trim() ||
    manifest.name?.trim() ||
    opts.id;
  const version = manifest.version?.trim() || "";
  const primary =
    (manifest.minecraft?.modLoaders ?? []).find((l) => l.primary) ??
    manifest.minecraft?.modLoaders?.[0];
  if (!primary?.id) {
    throw new Error("manifest.json has no minecraft.modLoaders entry");
  }
  const { loader, version: loaderVersion } = parseLoaderId(primary.id);
  const mc = manifest.minecraft?.version;
  if (!mc) {
    throw new Error("manifest.json missing minecraft.version");
  }

  return `# Modpack profile - generated by import-curseforge-manifest.mjs
# Base mods come from manifests/${opts.id}/manifest.json (no [pack] zip).

instance_name = ${escapeTomlBasic(instance)}
display_name = ${escapeTomlBasic(manifest.name?.trim() || instance)}
display_version = ${escapeTomlBasic(version)}

minecraft = ${escapeTomlBasic(mc)}
loader = ${escapeTomlBasic(loader)}
loader_version = ${escapeTomlBasic(loaderVersion)}
fallback_ram_gb = ${opts.fallbackRamGb}
min_ram_gb = ${opts.minRamGb}

[server]
name = ${escapeTomlBasic(opts.serverName)}
address = ${escapeTomlBasic(opts.serverAddress)}

# --- Always installed (from CurseForge manifest + manual extras) -------------

${block}

${EXTRAS_HINT}
`;
}

function replaceManifestBlock(existing, block) {
  const beginIdx = existing.indexOf(BEGIN);
  const endIdx = existing.indexOf(END);
  if (beginIdx === -1 || endIdx === -1 || endIdx < beginIdx) {
    throw new Error(
      `Existing pack TOML is missing ${BEGIN} / ${END} markers.\n` +
        `Add them around the generated [[required_curseforge]] block, or delete the TOML and re-run.`,
    );
  }
  const endLineEnd = endIdx + END.length;
  return (
    existing.slice(0, beginIdx) +
    block +
    existing.slice(endLineEnd)
  );
}

function updateIndex(packId) {
  let text = existsSync(INDEX_PATH)
    ? readFileSync(INDEX_PATH, "utf8")
    : `# Curated modpack index - fetched from GitHub at startup; embedded copy is fallback.\ndefault = "${packId}"\n`;

  const entryRe = new RegExp(
    `\\[\\[packs\\]\\]\\s*\\nid\\s*=\\s*"${packId}"\\s*\\npath\\s*=\\s*"[^"]*"`,
    "m",
  );
  const entry = `[[packs]]\nid = "${packId}"\npath = "packs/${packId}.toml"`;
  if (entryRe.test(text)) {
    text = text.replace(entryRe, entry);
  } else {
    if (!text.endsWith("\n")) {
      text += "\n";
    }
    text += `\n${entry}\n`;
  }
  writeFileSync(INDEX_PATH, text);
}

async function main() {
  let args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (err) {
    console.error(err.message ?? err);
    usage(1);
  }

  const manifestPath = resolve(args.manifest);
  if (!existsSync(manifestPath)) {
    console.error(`Manifest not found: ${manifestPath}`);
    process.exit(1);
  }

  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.manifestType && manifest.manifestType !== "minecraftModpack") {
    console.error(
      `Unexpected manifestType: ${manifest.manifestType} (expected minecraftModpack)`,
    );
    process.exit(1);
  }

  const files = (manifest.files ?? []).filter((f) => f.required !== false);
  const projectIds = [...new Set(files.map((f) => f.projectID).filter(Boolean))];

  const apiKey = readApiKey();
  let names = new Map();
  if (!apiKey) {
    console.warn(
      "Warning: no CurseForge API key — comments will use project IDs only.\n" +
        "Set CURSEFORGE_API_KEY or create src-tauri/curseforge_api_key.",
    );
  } else {
    console.log(`Resolving names for ${projectIds.length} CurseForge project(s)…`);
    names = await fetchModNames(apiKey, projectIds);
  }

  const destDir = join(MANIFESTS_DIR, args.id);
  mkdirSync(destDir, { recursive: true });
  const destManifest = join(destDir, "manifest.json");
  copyFileSync(manifestPath, destManifest);
  console.log(`Wrote ${destManifest}`);

  const block = buildManifestBlock(files, names);
  mkdirSync(PACKS_DIR, { recursive: true });
  const packPath = join(PACKS_DIR, `${args.id}.toml`);

  if (existsSync(packPath)) {
    const existing = readFileSync(packPath, "utf8");
    const updated = replaceManifestBlock(existing, block);
    writeFileSync(packPath, updated);
    console.log(`Updated manifest block in ${packPath}`);
  } else {
    const toml = buildSkeletonToml(args, manifest, block);
    writeFileSync(packPath, toml);
    console.log(`Created ${packPath}`);
  }

  updateIndex(args.id);
  console.log(`Updated ${INDEX_PATH}`);
  console.log(
    `Done: ${files.length} required mod(s) imported for pack "${args.id}".`,
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
