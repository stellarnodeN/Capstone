// Core state modules - consolidated and optimized
pub mod admin;      // Protocol administration
pub mod study;      // Study management 
pub mod nft;        // Consent NFT tracking
pub mod submission; // Data submission tracking
pub mod rewards;    // Reward distribution
pub mod survey;     // Survey schema and stats

// Re-export all state structs
pub use admin::*;
pub use study::*;
pub use nft::*;
pub use submission::*;
pub use rewards::*;
pub use survey::*;
