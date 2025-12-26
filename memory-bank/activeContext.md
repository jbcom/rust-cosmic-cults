# Active Context: Session 2025-12-25

## Current Focus
Reconciling the codebase at the new permanent location: `https://github.com/arcade-cabinet/cosmic-cults`.

## Recent Decisions
- **Upgraded to Bevy 0.17**: Migrated from 0.16 to 0.17 to leverage the latest features and ecosystem compatibility.
- **Massacred Custom Code**: Replaced custom AI, Physics, and Selection logic with `big-brain`, `avian3d`, and built-in `bevy_picking`.
- **Consolidated Monorepo**: Unified all game logic into the `cosmic-cults` crate.
- **Migrated to New Remote**: Set up `upstream` remote and preparing a pull request to reconcile changes.

## Next Steps
- Create PR on the new repository.
- Ensure all CI/CD workflows are updated for the new location.
- Verify full WASM build stability.
