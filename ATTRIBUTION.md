# Attribution

3D models in the object library (`library-assets/`, downloaded by `pnpm assets:fetch`) come from
**NVIDIA SimReady Assets** ([nvidia/simready-assets](https://huggingface.co/datasets/nvidia/simready-assets)),
licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The GLB conversions are from
[codepawl/simready-glb](https://huggingface.co/datasets/codepawl/simready-glb), pinned to a fixed revision
recorded in `packages/asset-catalog/src/catalog.json`.

The models are not modified by IntraCraft. Each manifest entry repeats the licence and credit.

## Rust platform (`platform/`)

`platform/` is [BIMCraft](https://github.com/IPedrax/BIMCraft) by IPedrax and the BIMCraft contributors, brought in as
a git subtree (BIMCraft commit `0733fd4`, [ADR-0020](docs/en/decisions/ADR-0020%20Rust%20Platform%20on%20BIMCraft%20with%20Three%20Modules.md)).
It is licensed MIT OR Apache-2.0 (`platform/LICENSE-MIT`, `platform/LICENSE-APACHE`); its notices are in
`platform/NOTICE`, which every package of the application must ship. Its Revit reader (`platform/crates/rvt`) is
Apache-2.0 only and its DWG bridge uses acadrust (MPL-2.0, unmodified); both are optional features. The IBM Plex fonts
in `platform/assets/fonts` are under the SIL Open Font License 1.1. Files of BIMCraft that IntraCraft changes are listed
in `platform/INTRACRAFT-CHANGES.md`.

`platform/vendor/` holds the unmodified sources of every Rust crate the build uses (`cargo vendor`), each with its own
licence file. IntraCraft's own crates (`platform/sim/`, `platform/apps/intracraft*`) are MIT OR Apache-2.0, like
BIMCraft.
