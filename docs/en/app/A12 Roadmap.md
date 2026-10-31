---
title: A12 Roadmap
lang: en
tags: [app, roadmap, plan]
---

🌐 **EN** · [[A12 Hoja de ruta|ES]] · [[A12 Phasenplan|DE]] · [[Home (EN)|Index]]

# A12 Roadmap

Order chosen to attack the biggest risk first (chain physics) and to deliver each conveyor end to end. Sizes in Claude Code work sessions: S ≈ 1–3, M ≈ 4–8, L ≈ 8–15.

## MVP phases

| Phase | Goal                  | Deliverables                                                                                                                         | Exit criterion                                       | Size | Risk   | Status (2026-10-31)                                                                                                                         |
| ----- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------- | ---- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 0     | Ground ready          | Repo, pnpm workspaces, AGENTS.md, docs, glossary, 3 reference models                                                                 | Tests run, CI passes                                 | S    | Low    | 🟡 Scaffold done; reference models and git repo pending                                                                                     |
| 1     | Reduce uncertainty    | S1 physics spike, S2 3D viewer with instanced rollers and gizmos, S3 minimal DES (Source → track → Sink)                             | S1 criteria met, engine decision recorded            | M    | High   | 🟢 S1 met with Jolt + plan B ([[ADR-0002 Physics Engine Candidate]], P1–P11); viewer started                                                |
| 2     | Core and library      | core-model (units, ports, connections, validation), definition schema, behaviour registry, pallet types, friction table, file format | Source → Sink saved and reopened; validations warn   | M    | Medium | 🟢 Schemas, registry, commands with undo, project files (open/save)                                                                         |
| 3     | Roller conveyor (2D)  | Definition, geometry, DES behaviour (kinematics, ZPA), sensors, stops, guides in 2D, basic controller, CLI                           | T1, T2; 3D animation of the 2D run                   | L    | Medium | 🟡 2D flow, source/sink, photocell (T6), If/Then controller, run in the editor; ZPA, stops, guides pending                                  |
| 4     | Physics base          | World, kinematic rollers, compound pallet, raycast sensors, stops, guides                                                            | S1 a, c, d, g inside the product; 2D/3D consistency  | L    | High   | 🟢 World built from the layout, raycast photocells, P6 met; lifts and gravity line in 3D ([[ADR-0016 Physics World Built from the Layout]]) |
| 5     | Chain conveyor        | Definition, DES with slip, physics with chosen plan                                                                                  | Pallet speed = chain ±3 %; accumulation with slip    | M    | High   | 🟡 3D with plan B (T1 on chains ±1 %); slip with accumulation pending                                                                       |
| 6     | Chain turntable       | Angular profile, stations, interlocks, ports per station                                                                             | T3 (6.5 s) and S1-e                                  | M    | Medium | 🟡 2D cycle done (ADR-0008); physics pending                                                                                                |
| 7     | Roller turntable      | Rollers on platform, locked during turn                                                                                              | Same tests as phase 6                                | S–M  | Low    | 🟡 2D cycle done (ADR-0008); physics pending                                                                                                |
| 8     | Roller→chain transfer | Liftable chains, interlocks, gap validation, both modes                                                                              | T4 (7.4 s) and 3D transfer without losing the pallet | L    | High   | 🟡 2D cycle done (ADR-0009); S05 rewrite to the lifting roller table proposed; physics pending                                              |
| 9     | Integration           | Reference layout with all five, mode switch, reference-model comparison, performance, docs, packaging                                | MVP definition of done ([[A01 Product Overview]])    | L    | Medium | 🟡 2D reference layout with all five runs (REF-5)                                                                                           |

🟢 done · 🟡 in progress · ⚪ not started

**Why this order:** the roller conveyor is the base of the others; chains concentrate the physics risk; turntables reuse rollers and chains; the transfer depends on everything before (rollers, chains, stops, sensors).

## AI-ready from the start

Phase 2 also delivers the command and query layer (every UI action is a command); after phase 3 the MCP server (stdio); after the MVP the embedded chat with own gateway and token — see [[A15 AI Agent Integration]].

## Next concrete steps

1. `git init`, first commit, push to a remote so CI runs.
2. ~~Spike S1 with Jolt~~ (done 2026-10-30: all criteria met, [[ADR-0002 Physics Engine Candidate]] accepted). The physics world built from a layout (phase 4) followed on 2026-10-31 ([[ADR-0016 Physics World Built from the Layout]]). Next: turntables and the transfer table in 3D.
3. ~~Spike S3: Source → roller track → Sink in the DES engine~~ (done 2026-10-30, [[ADR-0007 2D Flow Model Without Accumulation]]); T6 needs the photocell.
4. Answer the remaining questions in [[Open Questions]] (orientation, pallet type and roller sizes answered on 2026-10-29).

## IntraScript (ADR-0018)

The owner brought user code forward on 2026-10-31: [[A17 IntraScript Language]] and the information objects [[S18 Variable]] to [[S22 Area]].

| Step | Goal               | Deliverables                                                                                  | Exit criterion                                   | Size | Status (2026-10-31)                  |
| ---- | ------------------ | --------------------------------------------------------------------------------------------- | ------------------------------------------------ | ---- | ------------------------------------ |
| 1    | Specification      | A17, S18–S22, [[ADR-0018 IntraScript Scripting Language]]                                     | Owner confirms (Q20 in [[Open Questions]])       | S    | 🟡 Written, waiting for confirmation |
| 2    | Language core      | `packages/script`: lexer, parser, type and unit checker, interpreter with waiting activations | Examples of A17 run; exact unit and wait results | M    | ⚪                                   |
| 3    | Model and commands | `content`, `parentId`, `info.*` definitions, `script.*`, `data.*`, `layout.setParent`         | Undo, log and validations for all of them        | M    | ⚪                                   |
| 4    | 2D execution       | Events, waits, members from definitions, pallet actions, exit decisions                       | Scenarios IS-1 to IS-3, determinism              | L    | ⚪                                   |
| 5    | Editor             | Plan icons, code panel, table grid, console                                                   | A model with code built and run from the editor  | M    | ⚪                                   |
| 6    | 3D execution       | Code on the controller cycle, teleport moves                                                  | IS-1 to IS-3 agree with 2D within ±50 ms         | M    | ⚪                                   |

## After the MVP (suggested order)

1. Mid-run mode switch. 2. Automatic 2D↔3D calibration. 3. Curves, diverters, lifts, shuttles. 4. Boxes and totes. 5. Failures (MTBF/MTTR) and energy. 6. ~~User scripting~~ (brought forward: IntraScript, [[ADR-0018 IntraScript Scripting Language]]). 7. Experiments, replications, optimisation. 8. OPC UA / PLC. 9. Licensed manufacturer 3D models and DXF import. 10. Desktop version (Tauri).

## Related

[[A01 Product Overview]] · [[A11 Testing and Validation]]
