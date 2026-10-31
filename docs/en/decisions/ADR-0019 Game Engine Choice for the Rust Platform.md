---
title: ADR-0019 Game Engine Choice for the Rust Platform
lang: en
tags: [decision, architecture, rust, rendering, physics, pivot]
status: proposed
date: 2026-10-31
---

# ADR-0019 Game Engine Choice for the Rust Platform

- **Status:** Proposed on 2026-10-31 at the owner's request. Depends on [[ADR-0018 Pivot to a Simulation Module on a Rust BIM Platform]].
- **Date:** 2026-10-31

## Context

The owner asked whether a Rust game engine should be part of the new platform, and which one: **Bevy**, **Fyrox** or **Godot + Rust**. A game engine could give the 3D simulation view better rendering (lights, shadows, materials), native glTF/GLB loading for the catalogue models ([[A16 Asset Catalog]]), and a ready-made frame loop.

The platform already has, from BIMCraft: an egui desktop (`eframe` 0.36) with its own wgpu 3D view, plans, sections, sheets, the command layer and the CLI/MCP server. The simulation needs:

1. A **2D DES run much faster than real time**, headless, from the CLI and in tests.
2. A **3D physics run** with a fixed step of 1/120 s, **deterministic** for the same layout and seed, also runnable headless (CI, scenarios, agents).
3. Surface velocity for chains (Rapier `tangent_velocity`, [[ADR-0018 Pivot to a Simulation Module on a Rust BIM Platform]] §3).
4. A 3D view that draws hundreds of pallets and thousands of rollers smoothly, with the catalogue's GLB models.
5. The rules carried over from BIMCraft: never crash, `unsafe_code = "forbid"`, the core without UI, everything is a command.

## Status of the candidates (checked 2026-10-31)

| Engine           | Version                                                                    | Licence    | Architecture                                   | Physics                                                   | UI                                  | Editor                    |
| ---------------- | -------------------------------------------------------------------------- | ---------- | ---------------------------------------------- | --------------------------------------------------------- | ----------------------------------- | ------------------------- |
| **Bevy**         | 0.19 (June 2026); breaking release about every 3 months                    | MIT/Apache | Pure ECS, data-oriented, parallel systems      | `bevy_rapier3d` 0.33 or Avian 0.5 (plugins)               | `bevy_egui` 0.42 (egui inside Bevy) | None official; code-first |
| **Fyrox**        | 1.0 stable (March 2026); new features on hold for a few months             | MIT        | Scene graph of nodes                           | Rapier built in                                           | Own `fyrox-ui` (not egui)           | Full native editor        |
| **Godot + Rust** | Godot 4.x (C++); `godot-rust` gdext 0.5 (2026), breaking from time to time | MIT        | Node tree in C++, Rust through GDExtension FFI | Jolt built in (default for new projects planned from 4.6) | Godot's own UI                      | Godot editor              |

## Evaluation

Scores 1 (poor) to 5 (good) for **this** product, not for games in general.

| Criterion                                              | Bevy   | Fyrox  | Godot + Rust | Why                                                                                                                                                                                 |
| ------------------------------------------------------ | ------ | ------ | ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fits BIMCraft (egui, wgpu, pure Rust)                  | 4      | 2      | 1            | Bevy renders with wgpu and runs egui through `bevy_egui`; Fyrox has its own UI and renderer; Godot is a C++ engine with its own editor and UI                                       |
| `unsafe` forbidden, never crash                        | 4      | 4      | 1            | Bevy and Fyrox are Rust; Godot needs FFI and two runtimes                                                                                                                           |
| Headless, faster-than-real-time runs (CLI, CI, agents) | 5      | 3      | 2            | Bevy's `MinimalPlugins` + schedule runner run without a window; Fyrox and Godot are built around their window and editor                                                            |
| Determinism (same seed, same result)                   | 4      | 3      | 2            | Bevy's schedules can be ordered explicitly and Rapier/Avian have `enhanced-determinism`; Godot's Jolt is deterministic per platform, but the C++/Rust boundary is harder to control |
| Many moving objects (pallets, rollers)                 | 5      | 3      | 4            | ECS is made for thousands of similar entities; Godot's servers are fast; Fyrox's scene graph is heavier per node                                                                    |
| Rendering quality (light, shadows, PBR, glTF)          | 4      | 4      | 5            | All three have PBR and glTF; Godot's renderer is the most mature                                                                                                                    |
| Editor                                                 | 2      | 5      | 5            | Bevy has none; but the platform's editor is the **BIM editor**, a game editor would duplicate it                                                                                    |
| API stability                                          | 2      | 5      | 3            | Bevy breaks every ~3 months; Fyrox 1.0 is stable; gdext still breaks                                                                                                                |
| Size of ecosystem and community                        | 5      | 2      | 5            | Bevy is the largest Rust engine; Fyrox is mostly one main developer; Godot is large but mostly GDScript/C#                                                                          |
| **Total (of 45)**                                      | **35** | **31** | **28**       |                                                                                                                                                                                     |

## Decision (proposed)

1. **Bevy is the best of the three for this product.** It is pure Rust, uses the same graphics stack (wgpu) and the same UI toolkit (egui) as BIMCraft, runs headless, and its ECS suits a simulation with many similar objects. Fyrox is a strong second if a stable API matters more than integration; Godot + Rust is not recommended (C++ core, FFI, its editor and UI would compete with the BIM editor).
2. **Bevy is used for the 3D simulation viewer only, not as the base of the whole application.** The BIM editor stays BIMCraft's eframe app. Rewriting its ~48 000 lines of UI on Bevy would throw away the main reason for the pivot (reuse BIMCraft) and tie the whole product to Bevy's quarterly breaking changes.
3. **The simulation engines do not depend on Bevy.** `sim-des`, `sim-physics` (Rapier used directly, not through `bevy_rapier3d`) and `sim-control` stay plain Rust crates: deterministic, headless, tested without any engine, as A18 already plans. Bevy only draws snapshots. If Bevy is ever dropped, nothing in the simulation changes.
4. **How the viewer runs.** Bevy and eframe each want to own the window event loop, so the viewer is a **separate process** (`intracraft-viewer`) started by the desktop app. It reads the same project file and receives run snapshots through BIMCraft's existing control port (`bridge.rs`). It shows the 3D run with lights, shadows and the catalogue's GLB models, and uses `bevy_egui` for its own small panels (play, pause, speed, statistics of the selected object). Selection goes back to the editor through the same port.
5. **A gate before adopting it** (new spike R5b in [[A18 Rust Implementation Plan]]):
   - Same scene in the BIMCraft wgpu view (with instanced pallets and rollers) and in a Bevy viewer: 200 pallets, 2 000 rollers, 20 conveyors.
   - Measure frames per second on an ordinary laptop with integrated graphics, start-up time, binary size, and the effort to load the catalogue's GLB models.
   - If the BIMCraft view reaches 60 fps and loads the GLB models with little work, **stay without a game engine** (fewer dependencies, one process). Otherwise adopt the Bevy viewer.
6. **Version pinning.** If adopted, Bevy is pinned to one release and upgraded at most twice a year, in its own pull request, following the version rules of [[ADR-0001 Toolchain and Versions]].

## Alternatives

| Option                                         | Consequences                                                                                                             |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **A. Bevy as a separate 3D viewer (proposed)** | Best 3D view for the run, engines stay engine-free, BIM editor untouched; cost: a second process and a snapshot protocol |
| B. No game engine                              | One process, fewest dependencies; we must add instancing, lights and glTF loading to BIMCraft's own view                 |
| C. Whole application on Bevy                   | One modern stack; rewriting BIMCraft's UI, losing upstream updates, quarterly breaking changes everywhere                |
| D. Fyrox (viewer or whole app)                 | Stable API and a full editor; its own UI toolkit and renderer, duplicated editor, small community                        |
| E. Godot + Rust                                | Best renderer and editor; C++ core, FFI, `unsafe`, two languages, the Godot editor competing with the BIM editor         |

## Consequences

- No change to phases R0–R4 of [[A18 Rust Implementation Plan]]; spike R5b is added before R6.
- The asset catalogue's GLB models get a native loader (Bevy's glTF) in the viewer; the BIM side keeps importing them as family meshes.
- New dependency (if adopted): `bevy` and `bevy_egui`, both MIT/Apache; licences of their transitive dependencies are checked in R5b.

## Related

- [[ADR-0018 Pivot to a Simulation Module on a Rust BIM Platform]] · [[A17 BIMCraft Integration Analysis]] · [[A18 Rust Implementation Plan]] · [[ADR-0002 Physics Engine Candidate]] · [[A16 Asset Catalog]]
- Sources: [Bevy news (0.19)](https://bevy.org/news/) · [Bevy fixed timestep example](https://docs.rs/bevy/0.17.2/src/fixed_timestep/fixed_timestep.rs.html) · [Bevy `MinimalPlugins`](https://dev-docs.bevyengine.org/bevy/struct.MinimalPlugins.html) · [bevy_egui](https://crates.io/api/v1/crates/bevy_egui) · [Rapier with Bevy](https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy) · [Avian 3D](https://docs.rs/avian3d/latest/avian3d) · [Fyrox 1.0.0](https://fyrox.rs/blog/post/fyrox-game-engine-1-0-0/) · [Fyrox 1.0 (i-programmer)](https://www.i-programmer.info/news/144-graphics-and-games/19015-fyrox-game-engine-reaches-version-100.html) · [Fyrox README](https://docs.rs/crate/fyrox/0.24.0/source/README.md) · [godot-core releases](https://lib.rs/crates/godot-core) · [Godot: using Jolt Physics](https://docs.godotengine.org/en/stable/tutorials/physics/using_jolt_physics.html) · [Godot proposal: Jolt as default](https://github.com/godotengine/godot-proposals/issues/12289)
