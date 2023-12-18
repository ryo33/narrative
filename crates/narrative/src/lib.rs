pub mod async_step;
pub mod async_story;
pub mod environment;
mod independent_type;
pub mod step;
pub mod story;

pub use async_trait::async_trait;
pub use independent_type::IndependentType;
pub use narrative_macros::story;
pub mod serde {
    pub use serde::*;
}

pub struct RunAllError<T, E: std::error::Error> {
    pub step_id: T,
    pub error: E,
}
