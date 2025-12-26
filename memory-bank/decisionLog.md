# Decision Log

## 2025-12-25: Standardize on Ecosystem Crates
- **Decision**: Violently remove custom "from-scratch" logic for AI, Physics, and Selection.
- **Rationale**: There is no justification for custom implementations when high-quality ecosystem crates like `big-brain`, `avian3d`, and `bevy_picking` already exist. This reduces maintenance burden and leverages community-tested code.

## 2025-12-25: Migrate to Bevy 0.17
- **Decision**: Skip 0.16 and move directly to 0.17.
- **Rationale**: Stay on the bleeding edge for a compiled game. 0.17 provides significant performance improvements and cleaner APIs (especially for picking).

## 2025-12-25: Unified Monorepo
- **Decision**: Consolidate all crates into a single `cosmic-cults` crate.
- **Rationale**: Simplifies dependency management and development workflow for a single game project.
