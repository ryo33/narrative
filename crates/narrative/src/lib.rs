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

pub mod prelude {
    pub use crate::step::Run as _;
    pub use crate::step::RunAsync as _;
    pub use crate::story::StoryContext as _;
}

pub struct RunAllError<T, E: std::error::Error> {
    pub step_id: T,
    pub error: E,
}
