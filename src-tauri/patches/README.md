# Crate patches

Only the unified diffs are committed (~5 KB). Full crate trees are **not**.

| File | Change |
|------|--------|
| `lighty-modsloader+26.5.12.patch` | ForgeCDN fallback when `downloadUrl` is null; skip CF dep walk for pinned files |
| `lighty-launch+26.5.12.patch` | ForgeCDN fallback, hide Windows consoles, re-verify modpack files, safer downloads, NeoForge processor artifact check, pre-launch hook |
| `lighty-java+26.5.12.patch` | Detach JVM so closing the launcher doesn't kill Minecraft |

Apply before any `cargo` / `tauri` command (Cargo must see `patched/` at resolve time):

```bash
node scripts/apply-patches.mjs
```

`pnpm tauri:dev` / `pnpm tauri:build` run this automatically.

Fallback URL (FlowUpdater):

`https://edge.forgecdn.net/files/{fileId[0..4]}/{fileId[4..]}/{fileName}`
