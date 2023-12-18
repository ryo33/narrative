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

// This will be removed when async fn in trait is stabilized. Then we do advance major version,
// 1.0.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

pub struct RunAllError<T, E: std::error::Error> {
    pub step_id: T,
    pub error: E,
}
