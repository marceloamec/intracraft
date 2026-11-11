---
title: A22 High-Bay Warehouse Wizard
lang: en
tags: [app, design, storage, wizard]
status: specified — not built yet
updated: 2026-11-10
---

# A22 High-Bay Warehouse Wizard

A window of the **Design module** that asks for the size of a high-bay warehouse and places it on the plan: N storage
aisles ([[S26 Storage Aisle]]) side by side and, in front of each aisle, the **chain conveyors** its stacker crane picks
up from and sets down on. The front zone (loop, entries, exits) is not made by the wizard: it is drawn with the library,
or taken from an example (`examples/layouts/hbw-*.intracraft.json`). Decision:
[[ADR-0033 Scope Extension High-Bay Warehouse and Stacker Cranes]].

The same function is a command line tool, so that agents and tests make the same warehouses:
`intracraft-cli warehouse … --out FILE.intracraft.json`.

## 1. What it asks

| Group         | Value                         | Default       | Notes                                                                         |
| ------------- | ----------------------------- | ------------- | ----------------------------------------------------------------------------- |
| Size          | Rack length                   | 20 m          | Gives the bays: length / bay pitch, rounded down                              |
|               | Rack height                   | 15 m          | Gives the levels: (height − first level) / level pitch, rounded down          |
|               | Number of aisles              | 2             | 1 … 20                                                                        |
|               | Available width (optional)    | –             | "Fit": the most aisles that fit in this width                                 |
|               | Depth                         | single        | single or double deep                                                         |
| Pallet        | Pallet type                   | EUR           | [[S10 Pallet Types]]                                                          |
|               | Load height                   | 1.2 m         | Level pitch = pallet + load + 0.25 m, rounded up to 50 mm (can be overridden) |
| Stations      | Heights with stations         | 1             | 1 … 3                                                                         |
|               | Height of each                | 0.6 m, …      | Conveying height of its chain conveyors                                       |
|               | Use of each                   | in and out    | in and out, in only, out only                                                 |
|               | Chain conveyor length         | 2.4 m         | Room for three EUR pallets across                                             |
| Stacker crane | Speeds, ramps, fork times     | S26 defaults  | Copied to every aisle                                                         |
| Operation     | Initial fill, retrieval mode… | S26 defaults  | Copied to every aisle                                                         |
| Place         | Where, and turned how         | free spot, 0° | The front of the first aisle                                                  |

## 2. What it shows while you type

- Bays, levels and **storage positions** in total.
- The warehouse's **length, width and height**; the width is aisles × module width + the gaps between aisles (0.2 m).
- Per aisle, an **estimate** of the crane's work:
  - the mean **single cycle** (storage only): the mean over all positions of 2 moves and 2 fork cycles;
  - the mean **combined cycle** (one storage and one retrieval), through the two representative points of the usual
    guideline for crane cycle times: (1/5 length, 2/3 height) and (2/3 length, 1/5 height), with 4 fork cycles;
  - **pallets per hour** in and out for both.
- A warning for every value [[S26 Storage Aisle]] would refuse.

The estimate is for sizing. The discrete simulation gives the result of the real plant, with its conveyors and waits.

## 3. What it places

With the first aisle's front at the chosen place, every next aisle one module width + 0.2 m to its right:

| Object                 | Per                    | Name                | Where                                                      |
| ---------------------- | ---------------------- | ------------------- | ---------------------------------------------------------- |
| Storage aisle          | aisle                  | `Aisle 01`, …       | Side by side                                               |
| Infeed chain conveyor  | aisle and "in" height  | `Aisle 01 in 1`, …  | In front of the left row, running into the rack, on `INn`  |
| Outfeed chain conveyor | aisle and "out" height | `Aisle 01 out 1`, … | In front of the right row, running away from it, on `OUTn` |

- Conveyors above the floor height stand on a raised base, as at an elevator.
- Every object gets the section `HBW` ([[ADR-0029 Sections, Addresses and Load-Unit Names]]), so code and tables can
  address the warehouse as a whole.
- Everything is placed in **one step**: one Undo takes the whole warehouse away.
- Afterwards each object is an ordinary library object: its attributes can be changed, and aisles added or deleted.

## 4. Commands

- `intracraft_model::warehouse::plan(spec)`: the numbers of §2 and the list of §3, without changing anything.
- `intracraft_model::warehouse::build(session, library, spec)`: places the list in one transaction through BIMCraft's
  commands (`component`, `modify`), as every other change of the model.
- `intracraft-cli warehouse --length 40 --height 20 --aisles 4 --deep 2 --station 0.6:in --station 4.6:out --out
  FILE.intracraft.json`: the same plan written as a layout file.
