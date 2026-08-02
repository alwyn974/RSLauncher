# Design - RSLauncher

<!-- Visual system: Minecraft-native. Durable rules; exact tokens live in src/assets/main.css (@theme). -->

## World

The launcher speaks Minecraft's own UI language, not a generic dark desktop app. Every control is drawn the way the game draws its menus: flat color fields, black 2px borders, inset bevels (light top-left, dark bottom-right), pixel lettering with a hard drop shadow, square corners everywhere.

## Non-negotiables

- **Square corners.** No `border-radius` on any control, panel, input, or avatar. Ever.
- **Bevels, not soft shadows.** Depth comes from inset highlight/shadow pairs (MC button grammar). No blurred drop shadows, no glow, no glass, no gradients except the flat shine strip inside progress fills.
- **Pixel font discipline.** Monocraft (self-hosted, `src/assets/fonts/`) for the logo, view headings, button labels, and nav. Body text, form labels, and help copy use the system sans; logs use monospace. Never body text in the pixel face.
- **Minecraft text shadow.** Pixel-font text carries `text-shadow: 2px 2px 0 rgba(0,0,0,.5)`-style hard offset shadow. Sans body text never does.
- **Color roles.** Green = the Play action and its launch progress fill only. Gold = highlights, confirmations, selected state, version info (MC's yellow text). Red = errors and destructive actions. Grays carry everything else. Accent color is never decoration.
- **Dirt background.** The app ground is the darkened tiled dirt texture (`src/assets/dirt.svg`), like the game's options screens, under a heavy dark overlay so panels read clearly.
- **Icons are authored pixel SVGs** drawn on a 16×16 grid, filled, single color. No unicode glyphs, no emoji, no stock icon library.

## Motion

150–200 ms state transitions only: hover/press on buttons, progress fill advance, log lines appearing. No load choreography, no decorative animation. The one expressive moment is the launch sequence itself (progress bar + streaming logs).

## States

Every interactive control ships default / hover / focus-visible / active / disabled. The Play button is disabled while a launch is in progress; progress and logs always reflect the real (mocked) launch state machine.

## Contrast

Body/secondary text ≥ 4.5:1 on the dark ground; gold and green reserved for large or short text where they hold ≥ 3:1.
