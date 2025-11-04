pub mod environment;
mod independent_type;
pub mod runner;
pub mod step;
pub mod story;
pub mod value;

pub use independent_type::IndependentType;
pub use narrative_macros::*;

/// Marker trait for types owned by a single story.
/// Due to the Orphan Rule, using `#[local_type_for]` on remote types (types from other crates)
/// will result in a compilation error.
/// Additionally, attempting to use `#[local_type_for]` on the same type for multiple stories
/// will cause conflicting trait implementations.
pub trait StoryOwnedType: std::fmt::Debug + Clone + serde::Serialize {}

pub mod serde {
    pub use serde::*;
}

pub mod prelude {
    pub use crate::step::Run as _;
    pub use crate::step::RunAsync as _;
    pub use crate::step::Step as _;
    pub use crate::step::StepArg as _;
    pub use crate::story::RunStory as _;
    pub use crate::story::RunStoryAsync as _;
    pub use crate::story::StoryConst as _;
    pub use crate::story::StoryContext as _;
    pub use crate::story::StoryContext as _;
}
