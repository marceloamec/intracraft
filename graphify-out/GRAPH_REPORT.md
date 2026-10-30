# Graph Report - Dematic game  (2026-10-29)

## Corpus Check
- 255 files · ~174,411 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 4, .css 2)

## Summary
- 470 nodes · 689 edges · 28 communities (25 shown, 3 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 10 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `vitest` - 14 edges
3. `EventQueue` - 12 edges
4. `scripts` - 11 edges
5. `SimTime` - 10 edges
6. `Vec3` - 9 edges
7. `DesSimulator` - 9 edges
8. `PhysicsBackend` - 9 edges
9. `Toolbar()` - 8 edges
10. `useEditorStore` - 7 edges

## Surprising Connections (you probably didn't know these)
- `results` --calls--> `runScenario()`  [EXTRACTED]
  apps/cli/src/index.ts → packages/scenarios/src/analytic.ts
- `RollerConveyorMesh()` --calls--> `rollerLayout`  [EXTRACTED]
  apps/editor/src/components/RollerConveyorMesh.tsx → packages/geometry/src/rollers.ts
- `palletFront()` --calls--> `positionAt()`  [EXTRACTED]
  apps/editor/src/components/RollerConveyorMesh.tsx → packages/sim-des/src/kinematics.ts
- `loopDuration()` --calls--> `timeToReach()`  [EXTRACTED]
  apps/editor/src/components/RollerConveyorMesh.tsx → packages/sim-des/src/kinematics.ts
- `App()` --calls--> `Toolbar()`  [EXTRACTED]
  apps/editor/src/App.tsx → apps/editor/src/components/Toolbar.tsx

## Import Cycles
- None detected.

## Communities (28 total, 3 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.08
Nodes (26): failed, formatResult(), logger, results, createLogger(), LEVEL_ORDER, LogEntry, Logger (+18 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (37): dependencies, @intracraft/core-model@workspace:*, @intracraft/geometry@workspace:*, @intracraft/library@workspace:*, @intracraft/sim-des@workspace:*, react-dom@19.3.1, @react-three/fiber@9.8.2, three@0.186.1 (+29 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (25): localizedTextSchema, affectsSchema, ComponentDefinition, componentDefinitionSchema, ParameterDefinition, parameterSchema, PortDefinition, portSchema (+17 more)

### Community 3 - "Community 3"
Cohesion: 0.10
Nodes (20): secondsToSimTime(), simTimeToSeconds(), US_PER_MS, US_PER_S, degToRad(), GRAVITY, MM_PER_M, mmToM() (+12 more)

### Community 4 - "Community 4"
Cohesion: 0.09
Nodes (27): description, devDependencies, eslint@10.13.0, @eslint/js@10.0.1, eslint-plugin-react-hooks@7.1.1, globals@17.13.0, prettier@3.9.10, @types/node@24.19.2 (+19 more)

### Community 5 - "Community 5"
Cohesion: 0.11
Nodes (21): react@19.3.1, App(), accel, diameter, height, length, loopDuration(), palletFront() (+13 more)

### Community 6 - "Community 6"
Cohesion: 0.16
Nodes (9): SimTime, compareEvents(), EventQueue, QueuedEvent, DesSimulator, EventHandler, EventTrace, SimEvent (+1 more)

### Community 7 - "Community 7"
Cohesion: 0.18
Nodes (15): Toolbar(), EN, ENGLISH_KEYS, Language, LANGUAGES, StringKey, translate(), TRANSLATED_KEYS (+7 more)

### Community 8 - "Community 8"
Cohesion: 0.14
Nodes (11): ADR-0002, Vec3, BodyDesc, BodyHandle, BoxShape, CylinderShape, PhysicsBackend, Pose (+3 more)

### Community 9 - "Community 9"
Cohesion: 0.14
Nodes (15): Connection, connectionSchema, Instance, instanceSchema, Language, Layout, layoutSchema, localize() (+7 more)

### Community 10 - "Community 10"
Cohesion: 0.12
Nodes (16): compilerOptions, exactOptionalPropertyTypes, isolatedModules, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+8 more)

### Community 11 - "Community 11"
Cohesion: 0.15
Nodes (15): dependencies, @intracraft/core-model@workspace:*, @intracraft/scenarios@workspace:*, description, devDependencies, tsx@4.23.15, @intracraft/core-model, name (+7 more)

### Community 12 - "Community 12"
Cohesion: 0.14
Nodes (13): jobs, test, name, on, pull_request, push, run, uses (+5 more)

### Community 13 - "Community 13"
Cohesion: 0.18
Nodes (13): dependencies, @intracraft/core-model@workspace:*, @intracraft/sim-des@workspace:*, zod@4.6.7, description, exports, @intracraft/core-model, @intracraft/sim-des (+5 more)

### Community 14 - "Community 14"
Cohesion: 0.20
Nodes (11): dependencies, @intracraft/core-model@workspace:*, zod@4.6.7, description, exports, @intracraft/core-model, zod, name (+3 more)

### Community 15 - "Community 15"
Cohesion: 0.18
Nodes (11): scripts, build, dev, format, format:check, format:docs, lint, scenario (+3 more)

### Community 16 - "Community 16"
Cohesion: 0.27
Nodes (7): Condition, conditionHolds(), evaluateRules(), Rule, SignalValues, SignalWrite, ZPA_RULES

### Community 17 - "Community 17"
Cohesion: 0.22
Nodes (9): dependencies, zod@4.6.7, description, exports, zod, name, private, type (+1 more)

### Community 18 - "Community 18"
Cohesion: 0.22
Nodes (9): dependencies, @intracraft/core-model@workspace:*, description, exports, @intracraft/core-model, name, private, type (+1 more)

### Community 19 - "Community 19"
Cohesion: 0.22
Nodes (9): dependencies, @intracraft/core-model@workspace:*, description, exports, @intracraft/core-model, name, private, type (+1 more)

### Community 20 - "Community 20"
Cohesion: 0.31
Nodes (7): alignTables(), displayWidth(), formatTable(), isTableLine(), listMarkdown(), separator(), splitRow()

### Community 21 - "Community 21"
Cohesion: 0.25
Nodes (7): ./tsconfig.base.json, compilerOptions, jsx, lib, types, extends, include

### Community 22 - "Community 22"
Cohesion: 0.29
Nodes (6): description, exports, name, private, type, version

### Community 24 - "Community 24"
Cohesion: 0.29
Nodes (6): description, exports, name, private, type, version

### Community 25 - "Community 25"
Cohesion: 0.33
Nodes (5): printWidth, proseWrap, singleQuote, tabWidth, trailingComma

## Knowledge Gaps
- **4 isolated node(s):** `ADR-0003`, `ADR-0003`, `ADR-0002`, `./tsconfig.base.json`
  These have ≤1 connection - possible missing edges. (Counts symbols only; 205 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `vitest` connect `Community 4` to `Community 0`, `Community 2`, `Community 3`, `Community 5`, `Community 6`, `Community 7`, `Community 9`, `Community 16`, `Community 23`?**
  _High betweenness centrality (0.184) - this node is a cross-community bridge._
- **What connects `ADR-0003`, `ADR-0003`, `ADR-0002` to the rest of the system?**
  _4 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.0753045404208195 - nodes in this community are weakly interconnected._
- **Why does `react` connect `Community 5` to `Community 1`, `Community 7`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.07017543859649122 - nodes in this community are weakly interconnected._
- **Why does `scripts` connect `Community 15` to `Community 4`?**
  _High betweenness centrality (0.029) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.07965860597439545 - nodes in this community are weakly interconnected._