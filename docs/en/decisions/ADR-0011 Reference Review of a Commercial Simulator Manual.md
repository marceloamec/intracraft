---
title: ADR-0011 Reference Review of a Commercial Simulator Manual
lang: en
tags: [decision, clean-room, flow]
status: accepted
date: 2026-10-30
---

# ADR-0011 Reference Review of a Commercial Simulator Manual

- **Status:** Accepted
- **Date:** 2026-10-30

## Context

The repository contains the introductory manual of a commercial discrete-event simulator (`Introduction to Tecnomatix Plant Simulation english .pdf`). The clean-room rule in `AGENTS.md` forbids reading or copying third-party simulation software, and allows references to other programs only to compare observable results. The project owner asked on 2026-10-30 to read the manual for ideas for new flow blocks and explicitly allowed it **for concepts only**.

## Decision

- The introductory manual (text only, 1,560 lines) was read once as a source of **general concepts**. No code, project file, graphic, name, trademark, text or parameter set was copied.
- Names, parameters, ranges and formulas are written from scratch in our specifications. Our block is called "Process station" (`flow.process_station`), not the other product's object name.
- The review is limited to that manual. Other documentation or software of the same vendor is not read without a new decision.

## What was taken (concepts only)

| Concept in the manual                                    | Our version                                                    |
| -------------------------------------------------------- | -------------------------------------------------------------- |
| A work station holds a part for a processing time        | [[S11 Process Station]]                                        |
| The time can be fixed or follow a distribution           | `fixed`, `normal`, `exponential` with the project's seeded RNG |
| Objects report time working, waiting, blocked as shares  | `utilization`, `idleShare`, `blockedShare`                     |
| Source, station, conveyor and drain as the minimal chain | Already covered by `flow.source`, conveyors and `flow.sink`    |

## What was only noted

Intermediate buffer with a capacity, flow distribution by exit strategy, failures and set-up, hierarchical sub-models with interfaces, occupancy charts, flow diagrams and reports. They are outside the MVP (`AGENTS.md`, "Qué no hacer") and are listed in [[Block Ideas]] for the owner to approve or discard.

## Consequences

- Any future block inspired by the manual needs its own specification and a mention here.
- If the owner prefers a stricter reading of the clean-room rule, the manual can be removed from the repository and S11 stays valid (it does not depend on it).
