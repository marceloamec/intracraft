---
title: ADR-0025 Simulation Modules on BIMCraft's Ribbon and Detailed Families
lang: en
tags: [decision, ui, bimcraft, geometry, rust]
status: accepted
date: 2026-11-02
---

# ADR-0025 Simulation Modules on BIMCraft's Ribbon and Detailed Families

- **Status:** Accepted on 2026-11-02, at the owner's request ("the modes should be integrated in the ribbon … the 3D models of the conveyors should appear with rollers, chains, steel profiles, legs"). Refines [[ADR-0020 Rust Platform on BIMCraft with Three Modules]] §7 and [[A09 Editor and Viewer]] §0.
- **Date:** 2026-11-02

## Context

The first IntraCraft window put a module selector above BIMCraft's window and drew each simulation module on a plan of its own, beside or instead of BIMCraft. The owner asked for the modules to live in BIMCraft's ribbon, the interface integrated rather than wrapped round BIMCraft. The families' forms were two boxes (a deck and a frame) per object.

## Decision

1. **BIMCraft gets a generic extension hook (`bimcraft_ui_egui::Extension`)**, with nothing of IntraCraft in it:
   - `tabs` and `ribbon`: an application adds ribbon tabs after BIMCraft's own and draws their panels with BIMCraft's `ribbon_panel` and `ribbon_button` (the button paints an icon the application gives);
   - `panel_title` and `panel`: a card beside the views, styled as BIMCraft's palettes;
   - `over_view`: it draws over the active view once BIMCraft has, placing points with `BimApp::screen` (plan) and `BimApp::screen3` (3D view);
   - while one of its tabs is picked, a selection doesn't raise the contextual Modify tab.
   `BimApp::show` is `show_with` and no extension; `extension_tab`, `pick_extension_tab` and `open_view` let the application read and set them.
2. **IntraCraft's three modules are three ribbon tabs**: Intralogistics (the library's objects, Detach), Discrete simulation and Physics simulation (Control, Speed, Clock). The simulation is drawn over BIMCraft's own floor plan and 3D view: each object tinted by its state on the plan, the pallets and their loads moving as boxes. IntraCraft's icons are its own outlines, drawn with BIMCraft's pen.
3. **A family's form may repeat (`repeat`: `count` copies, each `step` further)**, an array as BIM families have: the rollers of a conveyor at their pitch, its legs at most 2 m apart, its chain strands across its width. BIMCraft's project file goes from format 12 to 13 (no migration step: a file of 12 holds no repeat).
4. **The library's families draw detailed parametric forms** (`intracraft-model::forms`), our own geometry after the reference's 3D models: rollers on their axles, C-channel side frames, chain strands on their tracks with the drive sprockets and the loop, legs on adjustable feet with floor plates, a cross beam from 650 mm and a diagonal brace from 850 mm conveying height, the geared motor, the chain guard, the end stop; turntables on a base with their slewing ring; the transfer table's chains in the gaps between its rollers and its lifting frame; scissor lift tables; the elevator's masts, top beam and carriage; the gravity line's falling rollers, brake rollers, separator and end stop; the process station's gantry; the photocell's posts, heads and beam; the source's and sink's stands. Their materials (`IntraCraft frame`, `IntraCraft roller`…) are made with the families. Every family builds at every parameter's least and greatest value (a test).

## Consequences

- Plans, sections, the 3D view, schedules and IFC show conveyors as conveyors.
- A formula can't read a choice (a text parameter): forms that depend on one are drawn for its default (the drive on the right, under the deck; a roller deck on the elevator and the process station). A yes/no parameter per choice, or text comparisons in BIMCraft's formulas, would lift this.
- Pallets over the 3D view are painted after it, without its depth: a pallet shows in front of a mast that hides it. The Bevy viewer (ADR-0022) remains the way to a true 3D window of the physics run.
- The BIMCraft files changed are listed in `platform/INTRACRAFT-CHANGES.md`.
