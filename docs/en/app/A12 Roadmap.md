---
title: A12 Roadmap
lang: en
tags: [app, roadmap, plan]
---

🌐 **EN** · [[A12 Hoja de ruta|ES]] · [[A12 Phasenplan|DE]] · [[Home (EN)|Index]]

# A12 Roadmap

Order chosen to attack the biggest risk first (chain physics) and to deliver each conveyor end to end. Sizes in Claude Code work sessions: S ≈ 1–3, M ≈ 4–8, L ≈ 8–15.

## MVP phases

| Phase | Goal                  | Deliverables                                                                                                                         | Exit criterion                                       | Size | Risk   | Status (2026-10-28)                                                |
| ----- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------- | ---- | ------ | ------------------------------------------------------------------ |
| 0     | Ground ready          | Repo, pnpm workspaces, AGENTS.md, docs, glossary, 3 reference models                                                                 | Tests run, CI passes                                 | S    | Low    | 🟡 Scaffold done; reference models and git repo pending            |
| 1     | Reduce uncertainty    | S1 physics spike, S2 3D viewer with instanced rollers and gizmos, S3 minimal DES (Source → track → Sink)                             | S1 criteria met, engine decision recorded            | M    | High   | 🟡 Research done ([[T15 Physics Engine Research]]); viewer started |
| 2     | Core and library      | core-model (units, ports, connections, validation), definition schema, behaviour registry, pallet types, friction table, file format | Source → Sink saved and reopened; validations warn   | M    | Medium | 🟡 Schemas, registry, pallet and friction data exist               |
| 3     | Roller conveyor (2D)  | Definition, geometry, DES behaviour (kinematics, ZPA), sensors, stops, guides in 2D, basic controller, CLI                           | T1, T2; 3D animation of the 2D run                   | L    | Medium | 🟡 Definition JSON, kinematics, CLI                                |
| 4     | Physics base          | World, kinematic rollers, compound pallet, raycast sensors, stops, guides                                                            | S1 a, c, d, g inside the product; 2D/3D consistency  | L    | High   | ⚪                                                                 |
| 5     | Chain conveyor        | Definition, DES with slip, physics with chosen plan                                                                                  | Pallet speed = chain ±3 %; accumulation with slip    | M    | High   | ⚪                                                                 |
| 6     | Chain turntable       | Angular profile, stations, interlocks, ports per station                                                                             | T3 (6.5 s) and S1-e                                  | M    | Medium | ⚪ (T3 formula ✅)                                                 |
| 7     | Roller turntable      | Rollers on platform, locked during turn                                                                                              | Same tests as phase 6                                | S–M  | Low    | ⚪                                                                 |
| 8     | Roller→chain transfer | Liftable chains, interlocks, gap validation, both modes                                                                              | T4 (7.4 s) and 3D transfer without losing the pallet | L    | High   | ⚪ (T4 formula ✅)                                                 |
| 9     | Integration           | Reference layout with all five, mode switch, reference-model comparison, performance, docs, packaging                                | MVP definition of done ([[A01 Product Overview]])    | L    | Medium | ⚪                                                                 |

🟢 done · 🟡 in progress · ⚪ not started

**Why this order:** the roller conveyor is the base of the others; chains concentrate the physics risk; turntables reuse rollers and chains; the transfer depends on everything before (rollers, chains, stops, sensors).

## AI-ready from the start

Phase 2 also delivers the command and query layer (every UI action is a command); after phase 3 the MCP server (stdio); after the MVP the embedded chat with own gateway and token — see [[A15 AI Agent Integration]].

## Next concrete steps

1. `git init`, first commit, push to a remote so CI runs.
2. Spike S1 with Jolt (`jolt-physics`): tests a, b (plan B surface velocity), c, g; fall back to Rapier + plan A if needed. Close [[ADR-0002 Physics Engine Candidate]].
3. Spike S3: Source → roller track → Sink in the DES engine with T6.
4. Answer the remaining questions in [[Open Questions]] (orientation, pallet type and roller sizes answered on 2026-10-29).

## After the MVP (suggested order)

1. Mid-run mode switch. 2. Automatic 2D↔3D calibration. 3. Curves, diverters, lifts, shuttles. 4. Boxes and totes. 5. Failures (MTBF/MTTR) and energy. 6. User scripting. 7. Experiments, replications, optimisation. 8. OPC UA / PLC. 9. Licensed manufacturer 3D models and DXF import. 10. Desktop version (Tauri).

## Related

[[A01 Product Overview]] · [[A11 Testing and Validation]]
