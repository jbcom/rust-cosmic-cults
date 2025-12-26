pub mod assets;
pub mod units;
pub mod world;

// Re-export main plugins
pub use units::GameUnitsPlugin;
pub use world::GameWorldPlugin;

// Re-export common types
pub use assets::Cult;
pub use units::{Unit, Team, Leader, Health};
