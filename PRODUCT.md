# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Personal project: the author (and later friends) launching one fixed custom Minecraft modpack. Desktop app (Tauri webview shell), used in a gaming context - quick in, play, quick out.

## Product Purpose

RSLauncher is a minimal Minecraft launcher for a single custom modpack. It signs players in with their Microsoft account, keeps the modpack up to date, and launches the game. Success means: open → pick account → press Play → watch clear progress → game starts.

## Positioning

Deliberately smaller than general-purpose launchers (Prism, CurseForge, Modrinth App): no version browsing, no mod management, no instance system. One modpack, one Play button, the few settings players actually touch (RAM, resolution, JVM args).

## Operating Context

- Tauri 2 + Vue 3 + Tailwind 4 desktop app (Windows-first, `tauri.conf.json` window 1280×720).
- Microsoft (Xbox Live) authentication for Minecraft accounts.
- Multiple stored accounts; the player picks which one to play with.
- Launch pipeline: prepare → download/verify files → launch JVM → game running; the UI surfaces every step with progress (percent, current file, speed, ETA) and a game log viewer.

## Capabilities and Constraints

- Microsoft login button as the only auth path.
- Multiple Minecraft accounts stored locally; switchable; removable.
- Fixed custom modpack - no version or modpack selection UI.
- Settings: RAM allocation, game resolution (width/height, fullscreen), custom JVM arguments.
- Progress display for downloads and launch steps: percent, file count, current file, speed, ETA.
- Game log viewer (levels, auto-scroll, copy/clear).
- First build is UI-only with a mocked state layer; real backend wiring (OAuth, download, JVM launch in Rust) is a later phase. Mock event shapes must mirror future Tauri events.
- Updater plugin already configured (GitHub releases endpoint in `tauri.conf.json`).

## Brand Commitments

- Name: **RSLauncher** (window title and product name in `tauri.conf.json`).
- Visual world: Minecraft-native - the game UI's own grammar (pixel lettering, beveled block buttons, dark dirt/stone surfaces), confirmed with the owner.

## Evidence on Hand

None beyond the scaffold. No logo, screenshots, or copy assets exist; the build authors what it needs. Do not fabricate player counts, server names, or modpack contents.

## Product Principles

- One job, one button: Play is the hero; everything else is secondary.
- Never hide launch state: the player always sees what is happening and how long it takes.
- Speak Minecraft: the interface uses the game's own visual language, familiar to any player.
- Settings are few and plain: RAM, resolution, JVM args - nothing else.
