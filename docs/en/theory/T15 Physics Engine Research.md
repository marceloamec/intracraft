---
title: T15 Physics Engine Research
lang: en
tags: [theory, physics, research]
date: 2026-10-28
---

🌐 **EN** · [[T15 Investigación de motores de física|ES]] · [[T15 Recherche Physik-Engines|DE]] · [[Home (EN)|Index]]

# T15 Physics Engine Research

> **Rust platform (2026-10-31):** this research covers the JavaScript bindings used by the TypeScript reference. For the Rust platform the engine is Rapier (native), which has the surface velocity the JS binding lacked; Jolt has no maintained Rust binding ([[ADR-0019 Bevy Engine Feasibility]], [[ADR-0021 Game Engine Choice for the Rust Platform]]).

Research carried out on **2026-10-28** to prepare spike S1. Method: the npm packages were downloaded (not installed in the project), their TypeScript typings inspected, and default settings read by instantiating each engine in Node.js 24. Nothing from these packages was copied into IntraCraft.

## 1. Candidates and versions

| Engine | npm package                               | Version checked | Licence    | Language / build                                 |
| ------ | ----------------------------------------- | --------------- | ---------- | ------------------------------------------------ |
| Rapier | `@dimforge/rapier3d-compat`               | 0.21.0          | Apache-2.0 | Rust → WASM (base64-embedded)                    |
| Rapier | `@dimforge/rapier3d-deterministic-compat` | 0.21.0          | Apache-2.0 | Same, cross-platform deterministic build         |
| Jolt   | `jolt-physics`                            | 1.1.1           | MIT        | C++ → WASM (Emscripten), also multithread builds |

## 2. The deciding question: surface velocity in JavaScript

| Capability                    | Rapier JS 0.21.0                                                         | Jolt JS 1.1.1                                                                             |
| ----------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| Contact modification hook     | ❌ `PhysicsHooks` has only `filterContactPair`, `filterIntersectionPair` | ✅ `ContactListenerJS.OnContactAdded / OnContactPersisted(…, ioSettings)`                 |
| Per-contact surface velocity  | ❌ (the Rust `modify_solver_contacts` hook is not bound)                 | ✅ `ContactSettings.mRelativeLinearSurfaceVelocity` and `mRelativeAngularSurfaceVelocity` |
| Per-contact friction override | ❌ (only combine rules: Average, Min, Multiply, Max)                     | ✅ `ContactSettings.mCombinedFriction`, `mCombinedRestitution`                            |
| Contact filtering             | ✅                                                                       | ✅ `OnContactValidate`                                                                    |
| Plan B for chains             | Not possible → plan A or D                                               | **Feasible** (to confirm in S1)                                                           |

**Open points for S1 with Jolt:** sign/frame convention of the relative surface velocity (body 1 vs body 2); cost of a JS callback per contact per step (hundreds of contacts × 120 Hz); whether the callback order is deterministic; memory management of Emscripten objects (`Jolt.destroy`).

## 3. Measured default settings

| Setting                         | Rapier 0.21.0                            | Jolt 1.1.1                             | Why it matters                              |
| ------------------------------- | ---------------------------------------- | -------------------------------------- | ------------------------------------------- |
| Default time step               | `dt` = 1/60 s                            | set per `Update` call                  | IntraCraft uses 1/120 s                     |
| Solver iterations               | `numSolverIterations` = 4                | 10 velocity + 2 position steps         | Heavy pallets on many rollers (S1-a)        |
| Allowed penetration             | `normalizedAllowedLinearError` = 0.005 m | `mPenetrationSlop` = 0.02 m            | ±5 mm stop criterion (P1) → must be lowered |
| Speculative contact distance    | —                                        | `mSpeculativeContactDistance` = 0.02 m | Contact detection ahead of time             |
| Restitution threshold           | —                                        | `mMinVelocityForRestitution` = 1 m/s   | Impacts at 0.1–0.25 m/s would not bounce    |
| Default friction / combine rule | 0.5 / Average                            | geometric mean (documented default)    | Exact table values per material pair        |

## 4. Determinism

- **Rapier** ships a dedicated `-deterministic` build that guarantees **cross-platform** determinism (slower). The standard build is deterministic on the same machine only.
- **Jolt** supports deterministic simulation when bodies are added in the same order and the same API calls are made; **cross-platform** determinism requires a build with `CROSS_PLATFORM_DETERMINISTIC=ON` (the README of `jolt-physics` documents the flag). Whether the published npm build has it enabled must be checked in S1; otherwise only same-machine determinism is guaranteed.
- IntraCraft's test S1-g (same hash after 60 s) only requires same-machine determinism for the MVP.

## 5. Other observations

- Both run in a Web Worker (the plan's execution model).
- Jolt offers multithreaded WASM builds (needs `SharedArrayBuffer` → cross-origin isolation headers). Not needed for the MVP target (20 conveyors, 10 pallets).
- Rapier has a mature React integration (`@react-three/rapier`), but IntraCraft runs physics in a worker behind its own wrapper, so that advantage does not apply.
- three.js ships example adapters for both engines (`examples/jsm/physics/`), useful as reading material only.

## 6. Preliminary conclusion

Jolt is the **preferred candidate** for S1 because it is the only one whose JS binding exposes per-contact surface velocity and friction, which makes chain conveyors (plan B) and exact friction pairs possible without extra bodies. Rapier remains the fallback with plan A. The decision is recorded as *Proposed* in [[ADR-0002 Physics Engine Candidate]] and closes only after S1 passes.

## Sources

- npm: `@dimforge/rapier3d-compat` 0.21.0 (`dist/pipeline/physics_hooks.d.ts`, `package.json`), `jolt-physics` 1.1.1 (`dist/types.d.ts`, `README.md`, `package.json`).
- [Rapier JS `PhysicsHooks` interface](https://rapier.rs/javascript3d/interfaces/PhysicsHooks.html) · [Rapier collider active hooks (JS guide)](https://rapier.rs/docs/user_guides/javascript/collider_active_hooks/)
- [Jolt `ContactSettings` (C++ docs)](https://jrouwe.github.io/JoltPhysics/class_contact_settings.html) · [Jolt deterministic simulation](https://jrouwe.github.io/JoltPhysics/#deterministic-simulation)
- [Rapier deterministic build README](https://cdn.jsdelivr.net/npm/@dimforge/rapier3d-deterministic@0.19.3/README.md)

## Related

[[T12 Rollers and Chains in Physics]] · [[T11 Contact, Friction and Restitution]] · [[ADR-0002 Physics Engine Candidate]] · [[A07 Physics Engine Wrapper]]
