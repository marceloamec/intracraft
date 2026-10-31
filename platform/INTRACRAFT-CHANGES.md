# IntraCraft changes to BIMCraft

This folder is [BIMCraft](https://github.com/IPedrax/BIMCraft) (MIT OR Apache-2.0), brought into the IntraCraft repository
as a git subtree ([ADR-0020](../docs/en/decisions/ADR-0020%20Rust%20Platform%20on%20BIMCraft%20with%20Three%20Modules.md)).
Upstream changes come in with:

```sh
git subtree pull --prefix=platform https://github.com/IPedrax/BIMCraft.git main --squash
```

IntraCraft adds its own crates in `sim/` and its own apps in `apps/intracraft*`. Every BIMCraft file IntraCraft changes is
listed here, with why, so that each change is marked as Apache-2.0 §4(b) asks and an upstream merge knows what to keep.

| File                  | Change                                                                                                                    |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `.cargo/config.toml`  | Builds from the vendored sources in `vendor/` (source replacement); BIMCraft's `i18n` alias is kept                         |
| `Cargo.toml`          | `sim/*` added to the workspace members (IntraCraft's crates)                                                              |
