use crate::step::{StepId, Steps};

pub trait StoryExt: Sized {
    type Error: std::error::Error;
    // This is not &str for future extensibility.
    /// Returns the title of the story.
    fn story_title() -> String;
    /// Returns the identifier of the story.
    fn story_ident() -> &'static str;
    /// Returns the steps of the story.
    fn steps() -> Steps<Self, Self::Error>;
    /// Run all steps in the story. It's a shortcut for iterating over the steps.
    fn run_all(self) -> Result<(), RunAllError<Self>>;
}

// Fields are public because this type is just a replacement of (StepId, S::Error).
/// A pair of a step identifier and an error returned by the step.
pub struct RunAllError<S: StoryExt> {
    /// The identifier of the step that failed.
    pub step_id: StepId<S>,
    /// The error returned by the step.
    pub error: S::Error,
}
