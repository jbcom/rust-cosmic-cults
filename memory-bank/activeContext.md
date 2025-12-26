# Active Context: Session 2025-12-25

## Current Focus
Developing core gameplay mechanics: Combat, Resource Gathering, and HUD integration.

## Recent Decisions
- **Upgraded to Bevy 0.17**: Migrated from 0.16 to 0.17.
- **Massacred Custom Code**: Replaced custom AI, Physics, and Selection logic with `big-brain`, `avian3d`, and built-in `bevy_picking`.
- **Consolidated Monorepo**: Unified all game logic into the `cosmic-cults` crate.
- **Implemented Combat System**: Added `CombatStats`, `DamageEvent` (using Bevy's new `Message` system), and AI `AttackAction`.
- **Implemented Resource Gathering**: Added `GatherAction` and `NearResourceScorer` to `big-brain` AI.
- **Integrated Egui HUD**: Created a real-time HUD for player resources and unit selection details.
- **Fixed CI dependencies**: Updated `ci.yml` with necessary Linux system libraries for Bevy 0.17.
- **Shored up AI Movement**: Replaced direct transform manipulation with `LinearVelocity` based movement using `avian3d`.
- **Optimized AI Sensing**: Replaced O(N^2) loops with `avian3d` spatial queries for resource and enemy detection.
- **Established Issue Backlog**: Identified technical debt and future features, creating 9 detailed GitHub issues.

## Next Steps
- Merge PR #21 to establish the new baseline.
- Ensure all CI/CD workflows are passing in the new location.
- Begin implementation of resource gathering and expanded cult mechanics.
