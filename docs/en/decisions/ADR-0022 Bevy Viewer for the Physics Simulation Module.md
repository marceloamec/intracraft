---
title: ADR-0022 Bevy Viewer for the Physics Simulation Module
lang: en
tags: [decision, rendering, bevy, physics, rust]
status: accepted
date: 2026-10-31
---

# ADR-0022 Bevy Viewer for the Physics Simulation Module

- **Status:** Accepted on 2026-10-31 by the owner, after spike R5b on the owner's laptop. **Amended on 2026-11-09 by [[ADR-0026 Bevy and Avian for the Physics Simulation Module]]:** the viewer serves both simulation modules and runs its own simulation from the file; the physics engine is Avian, so it now depends on Bevy.
- **Date:** 2026-10-31
- **Closes:** spike R5b of [[A19 Rust Implementation Plan]]; confirms decision 2 of [[ADR-0021 Game Engine Choice for the Rust Platform]].

## Context

[[ADR-0021 Game Engine Choice for the Rust Platform]] proposed Bevy only as the viewer of the 3D run, after a spike comparing it with a viewer built the BIMCraft way (eframe and egui-wgpu with instancing). Its rule: if the BIMCraft-style view held 60 frames per second, the product would stay without a game engine.

## Measured on the owner's laptop (2026-10-31)

Intel Core Ultra 9 275HX, NVIDIA GeForce RTX 5070 Laptop GPU (Vulkan), Windows 11, release builds, 1 600 × 900 px, vsync off, 30 s each, the first 30 frames left out. Scene: 20 roller conveyors, 2 000 spinning rollers, 200 pallets with loads moving, the camera circling once.

| Item                             | BIMCraft-style (egui + wgpu)          | Bevy 0.20                                   |
| -------------------------------- | ------------------------------------- | ------------------------------------------- |
| Mean frame rate                  | 2 141 frames/s                        | 498 frames/s                                |
| Slowest 5 % of frames, at least  | 1 353 frames/s                        | 401 frames/s                                |
| Slowest frame                    | 3.9 ms                                | 4.2 ms                                      |
| Start-up to the first frame      | 0.57 s                                | 0.60 s                                      |
| Drawing                          | One sun, no shadows, no anti-aliasing | PBR, cascaded shadows, tonemapping, 4× MSAA |
| Release build on this laptop     | together with Bevy: 3 min 15 s        |                                             |
| Executable                       | 15 MB                                 | 71 MB                                       |
| Viewer code (no tests, comments) | 672 lines                             | 286 lines                                   |

Both pass the 60 frames/s gate by far: the gate alone would have kept the product without a game engine. The owner looked at both and preferred Bevy's picture.

**Finding: the catalogue models did not load in either viewer.** The GLB files of the asset catalogue ([[A16 Asset Catalog]]) are compressed: they require `EXT_meshopt_compression`, `KHR_mesh_quantization` and `EXT_texture_webp`. The `gltf` crate refuses them, and Bevy 0.20's loader did not finish loading them (the spike waited for the model and never started measuring; the numbers above are with box pallets in both). The TypeScript reference reads them through three.js's meshopt decoder.

## Decision

1. **The Physics simulation module draws its 3D run with Bevy**, as [[ADR-0021 Game Engine Choice for the Rust Platform]] decision 4 describes: a separate viewer process (`platform/apps/intracraft-viewer`) that the desktop starts, fed with run snapshots; the simulation engines (`intracraft-physics` on Rapier, `intracraft-des`) never depend on Bevy.
2. **Bevy is pinned** to one release (0.20.x now) and upgraded at most twice a year, in its own change. Its crates are vendored into `platform/vendor/` when the viewer enters the workspace (phase R6), like every other dependency ([[ADR-0020 Rust Platform on BIMCraft with Three Modules]] §7).
3. **The catalogue models are converted once**, when they are fetched: meshopt and quantization decoded, WebP textures turned into PNG, so that any Rust reader takes them. The tool is chosen in R6 (a Node step with glTF-Transform next to `pnpm assets:fetch`, or a Rust decoder in the viewer); until then the viewer draws parametric shapes.
4. The Design module keeps BIMCraft's own 3D view (plans, sections, the static 3D model); Bevy is only the run's viewer.

## Consequences

- R6 builds `intracraft-viewer` on Bevy; the spike's code (`spikes/r5b-viewer/viewer-bevy`) is its starting point.
- The vendored sources grow with Bevy's crates (about 360 crates in a Windows build) when R6 starts.
- The spike's report tool said "no game engine needed" from the gate alone; the owner's choice overrides it, for picture quality.

## Related

- [[ADR-0021 Game Engine Choice for the Rust Platform]] · [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] · [[A19 Rust Implementation Plan]] · [[A16 Asset Catalog]]
