pub mod step;
pub mod story;
pub use narrative_macros::story;

pub mod prelude {
    pub use crate::story::StoryExt as _;
}
