---
title: Block Ideas
lang: en
tags: [backlog, flow, not-approved]
updated: 2026-10-30
---

# Block Ideas

Flow blocks that are **not approved**: they are outside the MVP scope in `AGENTS.md`. Each needs the owner's decision, a specification (S-note), tests and the full library checklist ([[A05 Component Library]]) before any code. Concept source: [[ADR-0011 Reference Review of a Commercial Simulator Manual]].

| Idea                      | What it would do                                                                        | Why it could matter                                  |
| ------------------------- | --------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| Buffer (store)            | Holds several pallets up to a capacity, FIFO or LIFO, without conveyor geometry         | Absorbs a slow station; shows required storage size  |
| Flow splitter             | Sends pallets to one of several outputs by share, in turn, by attribute or by free path | Models routing without a turntable                   |
| Flow merger               | Joins several inputs into one output with a priority rule                               | Merge points are typical bottlenecks                 |
| Parallel stations         | Several process stations behind one entry with a distribution rule                      | Capacity planning of a work area                     |
| Station failures / set-up | Random breakdowns, repair time, change-over time between pallet types                   | Availability analysis; excluded: "fallos" in the MVP |
| Pallet attributes         | Source writes attributes; stations and splitters read them                              | Needed for routing by type or destination            |
| Occupancy chart           | Histogram of pallets in an object over time                                             | Reads buffer sizing at a glance                      |
| Flow diagram              | Arrow widths proportional to the pallets that took each path                            | Shows which route carries the load                   |
| Run report                | One table of all statistics of a run                                                    | Handover of results; ties in with the CLI            |
| Sub-model with interfaces | A reusable group of blocks with entry and exit ports                                    | Keeps large layouts manageable                       |

Suggested order if approved: pallet attributes → splitter and merger → buffer → report.
