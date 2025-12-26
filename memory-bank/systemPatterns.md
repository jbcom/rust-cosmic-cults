# System Patterns

## Architecture
The project follows Bevy's ECS pattern strictly. It is organized into a single workspace member `cosmic-cults` with submodules for assets, units, world, etc.

## Key Patterns
- **Observers**: Used for event-driven logic like selection and unit clicks.
- **Utility AI**: `big-brain` is used for unit decision making, following the Scorer/Action pattern.
- **Ecosystem Plugins**: Prefer mature community plugins over custom code.
- **Component-Driven Visuals**: Health bars, selection rings, and auras are separate entities parented to units, driven by component changes.

## Data Flow
1. **Input**: `leafwing-input-manager` and `bevy_picking` capture user intent.
2. **AI**: `big-brain` thinkers process the world state and choose actions.
   - **MoveToAction**: Follows `MovementPath` waypoints.
   - **GatherAction**: Extracts resources from `ResourceNode`s within range.
3. **Physics**: `avian3d` handles spatial simulation and collisions.
4. **World**: Procedural generation systems build the map and spawn entities (including resource nodes) at startup.
