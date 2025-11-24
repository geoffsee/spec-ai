pub mod agent;
pub mod bootstrap_self;
pub mod cli;
pub mod embeddings;
#[cfg(feature = "api")]
pub mod mesh;
pub mod spec;
#[cfg(feature = "api")]
pub mod sync;
pub mod test_utils;
pub mod tools;

pub use spec_ai_config::{config, persistence, types};
pub use spec_ai_policy::{plugin, policy};
