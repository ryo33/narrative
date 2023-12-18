use crate::step::StepId;

/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext: Sized {
    type Step: 'static;
    type StepIter: Iterator<Item = &'static Self::Step>;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_ident(&self) -> &'static str;
    /// Returns the steps of the story.
    fn steps(&self) -> Self::StepIter;
}

// Fields are public because this type is just a replacement of (StepId, S::Error).
/// A pair of a step identifier and an error returned by the step.
pub struct RunAllError<I: StepId, E> {
    /// The identifier of the step that failed.
    pub step_id: I,
    /// The error returned by the step.
    pub error: E,
}
