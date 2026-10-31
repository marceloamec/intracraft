---
title: ADR-0019 Bevy Engine Feasibility
lang: en
tags: [decision, architecture, stack, physics, research]
status: proposed
date: 2026-10-31
---

# ADR-0019 Bevy Engine Feasibility

- **Status:** Proposed on 2026-10-31. Feasibility study asked for by the owner; the owner decides. Recommendation: do not adopt Bevy for the MVP; revisit at the desktop phase or if a trigger below appears.
- **Date:** 2026-10-31

## Context

On 2026-10-31 the owner asked whether the Bevy game engine (<https://github.com/bevyengine/bevy>) could be used in IntraCraft, and for its pros and cons.

Adopting Bevy would change decision 4 of the MVP plan ("TypeScript everywhere, local web first; desktop with Tauri later") and the differentiator "runs in the browser, does not depend on a game engine" (plan §1). `AGENTS.md` asks to confirm such a change with the owner before acting.

State of IntraCraft on 2026-10-31:

- About 19,400 lines of TypeScript in 9 packages and 2 apps, about 4,800 of them tests.
- 2D mode: own discrete-event engine (`sim-des`, about 5,000 lines), reference tests T1–T5.
- 3D mode: Jolt (`jolt-physics` 1.1.1, WASM) behind `PhysicsBackend`; spike S1 passed every criterion (P1–P11, [[ADR-0002 Physics Engine Candidate]]); the world is built from the layout ([[ADR-0016 Physics World Built from the Layout]]).
- Editor: React + react-three-fiber; property panels generated from the JSON definitions; the simulation runs in a Web Worker.
- Command layer with Zod schemas, ready for the MCP server ([[ADR-0004 AI-Ready Command Layer and MCP]]).

## Findings (researched 2026-10-31)

| Topic             | Finding                                                                                                                                                                                 |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Language, licence | Rust; MIT or Apache-2.0 (compatible with the clean-room and licence rules)                                                                                                              |
| Version           | 0.20 on 2026-10-08; 0.19 on 2026-06-19; 0.18 on 2026-01-13; 0.17 on 2025-09-30. No 1.0 date; every release has a migration guide with breaking changes                                  |
| Editor            | No official editor yet. 0.20 adds "Feathers", an editor-oriented UI toolkit, and works on the BSN scene format; both described as groundwork for a future editor                        |
| Physics plugins   | Avian 0.8 (Bevy-native, needs Bevy 0.20); `bevy_rapier3d` 0.37.0 (2026-10-10, Bevy 0.20, Rapier 0.36.1)                                                                                 |
| Moving surface    | Available in Rust in both: Avian `ContactManifold.tangent_velocity` ("can be used to simulate effects such as conveyor belts"); Rapier `modify_solver_contacts` with a surface velocity |
| Jolt from Rust    | No maintained binding: `rolt` 0.3.1 (Jolt 5.0.0, May 2024) and `jolt-rust` (work in progress). No Bevy integration found                                                                |
| Determinism       | Avian lists "proper cross-platform determinism" as future work; Bevy 0.20 adds schedule randomization to find hidden ordering bugs                                                      |
| Web               | WebGL2 or WebGPU chosen at build time; the renderer is single-threaded on the web; a bare app measured about 26 MB of WASM after `wasm-opt` (Bevy 0.14, 2024)                           |

## Options

| Option                                   | What it means                                                                                                                  | Consequence                                                                                                                                                                                   |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A. Move everything to Bevy (Rust)        | Rewrite model, library, DES engine, physics world, controller, commands and editor in Rust; property panels in Bevy UI or egui | Redo spike S1 with Avian or Rapier (Jolt is not available); redo T1–T5, P1–P11 and the 2D/3D comparisons; a Bevy upgrade every 3–6 months; native desktop app from the same code              |
| B. Bevy only for the 3D view and physics | Keep TypeScript for model, DES, commands and React editor; a Bevy WASM module draws the 3D view and runs the physics           | Two languages and two toolchains (pnpm and cargo); layout, library and snapshot schemas defined twice (Zod and serde); a large WASM download; the TypeScript `sim-physics` world is rewritten |
| C. Keep the current stack (recommended)  | TypeScript, three.js and Jolt; revisit at the desktop phase ([[A12 Roadmap]] item 10)                                          | No rework; the measured results stay valid; the visual quality and the size of the largest layout are limited by three.js and by a single simulation worker                                   |

## Pros of Bevy for IntraCraft

1. **Speed.** Compiled Rust and a parallel ECS on desktop. Relevant for large plants (hundreds of conveyors, thousands of rollers).
2. **Moving surface in both Rust physics engines.** Avian and Rapier offer it in Rust; in the JS bindings only Jolt does. One language for simulation and drawing, no JS↔WASM boundary.
3. **Native desktop app** for Windows, macOS and Linux from one code base, plus a web build.
4. **Modern renderer:** physically based materials, shadows, realtime path tracing (Solari), mesh shaders; glTF/GLB loading built in (the catalogue models are GLB).
5. **Permissive licence**, no royalties.
6. **Strict typing and `cargo test`**, comparable to the current TypeScript and Vitest discipline.

## Cons of Bevy for IntraCraft

1. **The whole project is rewritten.** Every package, every test and every validated result; Bevy brings nothing to the 2D mode, which is event-driven, not frame-driven.
2. **Physics results are lost.** Jolt has no maintained Rust binding, so spike S1 (stop at ±5 mm, guide funnel, chain plan B, determinism) must be repeated with another engine and could fail on criteria Jolt already meets.
3. **Not stable.** Breaking changes in every release; the physics plugins and every other plugin must follow. Code produced by AI agents tends to use older Bevy APIs, so each step needs more correction rounds.
4. **No editor and weak form-style UI.** IntraCraft is an engineering editor (generated property panels, tables, library panel, three languages). React does this well today; in Bevy it would be built with egui or with the young Feathers toolkit.
5. **Weaker on the web.** Large download, single-threaded renderer, embedding next to HTML panels is awkward. This goes against "local web first" and the browser differentiator.
6. **Determinism is not guaranteed across machines** in Avian; the project treats a determinism failure as top priority.
7. **The AI command layer is redone:** Zod schemas, validation and the planned MCP server move to Rust.

## Decision (proposed)

- **Option C:** keep TypeScript, three.js and Jolt for the MVP.
- Revisit Bevy (option A) only if one of these happens:
  - layouts of the target size cannot run at real time in 3D after profiling;
  - Bevy reaches 1.0 or a stable release policy, with an official editor;
  - the desktop version (roadmap item 10) needs rendering or speed that Tauri with the web editor cannot give.
- If Rust is ever needed for speed alone, a Rust physics module compiled to WASM behind `PhysicsBackend` (plan D of [[T12 Rollers and Chains in Physics]]) is a smaller step than an engine change.

## Sources

- Bevy releases: <https://bevy.org/news/>; Bevy 0.20: <https://bevy.org/news/bevy-0-20/>; migration guide 0.19→0.20: <https://bevy.org/learn/migration-guides/0-19-to-0-20/>
- Avian `ContactManifold`: <https://docs.rs/avian3d/latest/avian3d/collision/contact_types/struct.ContactManifold.html>; `CollisionHooks`: <https://docs.rs/avian3d/latest/avian3d/collision/hooks/trait.CollisionHooks.html>
- `bevy_rapier3d` 0.37.0: <https://docs.rs/crate/bevy_rapier3d/latest>; Rapier hooks: <https://docs.rs/crate/rapier3d/latest/source/src/pipeline/physics_hooks.rs>
- Jolt in Rust: <https://docs.rs/crate/rolt/latest>, <https://github.com/Bantarus/jolt-rust>
- Avian README (determinism roadmap): <https://github.com/janhohenheim/avian/blob/main/README.md>
- WASM size: <https://docs.rs/crate/beetmash_template/0.0.6/source/README.md>; web threading: <https://github.com/bevyengine/bevy/pull/12205>
