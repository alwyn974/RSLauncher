# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Mod ignorelist

To keep personal mods from being removed by the orphan cleanup, create a
`mods-ignorelist.txt` file in either the RSLauncher data directory or an
instance directory. The launcher-level file applies to every instance; both
files are combined when an instance also has its own list.

Add one mod filename or wildcard pattern per line. Matching is
case-insensitive, `*` matches any number of characters, `?` matches one, and
lines starting with `#` are comments. A `.jar` entry also protects its
`.jar.disabled` counterpart.

```text
# Personal client-side mods
Essential-*.jar
voicechat.jar
```
