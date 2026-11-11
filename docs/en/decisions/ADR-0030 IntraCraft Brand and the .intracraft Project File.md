---
title: ADR-0030 IntraCraft Brand and the .intracraft Project File
lang: en
tags: [decision, brand, ui, file-format]
status: accepted
date: 2026-11-10
---

# ADR-0030 IntraCraft Brand and the .intracraft Project File

- **Status:** Accepted and built.
- **Date:** 2026-11-10

## Context

The Rust application ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]]) showed BIMCraft's brand: its logo and
name on the start page, egui's default window icon, no icon on `intracraft.exe`, and Save As offered `.bimcraft`. The
brand folder (`brand/`) held three logo proposals. The repository had no front page on GitHub.

The owner asked to:

1. Apply logo variant A, "rail" (`brand/logo/variants/a-rail.svg`): a navy tile with two white rails and an amber pallet.
2. Save projects as `.intracraft`, while `.bimcraft` files keep opening.
3. Write the repository's front page (README), giving BIMCraft full credit.

They chose, out of the alternatives offered:

- **Colours:** the logo and the icons only; the interface keeps BIMCraft's palette (not the navy bar and steel accent,
  nor a full palette).
- **README language:** English, as the reference documentation.

## Decision

- **A brand on BIMCraft's window.** `bimcraft-ui-egui` gains `Brand` (name, tagline, logo painter, source page, project
  file name and extension) and `BimApp::brand`, BIMCraft's own by default, so BIMCraft alone looks and saves as before.
  The start page draws the brand's logo, name, tagline and source link. Listed in `platform/INTRACRAFT-CHANGES.md`.
- **IntraCraft's brand** is `platform/apps/intracraft/src/brand.rs`: the rail logo drawn in code (for the start page,
  and with tiny-skia for pictures). The window's icon is drawn from it at start-up; `brand/export` holds the PNGs and
  `intracraft.ico`, written by `cargo run -p intracraft -- --write-brand`, and a test fails when they are out of date.
  `apps/intracraft/build.rs` builds the `.ico` into `intracraft.exe`, the way BIMCraft's `build.rs` does.
- **`.intracraft` is a BIMCraft project file under IntraCraft's name.** Same content, same `FORMAT` and migrations; only
  the extension differs. Save As proposes `.intracraft`; File > Open, the start page, a dropped file and the command line
  take `.intracraft`, `.bimcraft` and IFC. `.intracraft.json` stays the TypeScript reference's layout, placed in a new
  project when opened. The recovery copy keeps BIMCraft's name (`<project>.autosave.bimcraft`).
- **README.md** at the repository's root, in English, after BIMCraft's README, with a Credits section for BIMCraft.

## Consequences

- An upstream BIMCraft merge must keep `Brand` in `lib.rs` and `start.rs`.
- The installers and file associations (Windows `.wxs`, Linux MIME, macOS bundle) are BIMCraft's and still name
  `.bimcraft`; IntraCraft has no installer yet. When it gets one, it registers `.intracraft`.
- The interface's colours stay BIMCraft's; changing them later is a separate decision.

## Related

- [[ADR-0020 Rust Platform on BIMCraft with Three Modules]]
- [[ADR-0025 Simulation Modules on BIMCraft's Ribbon and Detailed Families]]
