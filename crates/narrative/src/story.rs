/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext: Sized {
    type Step: 'static;
    type StepIter: Iterator<Item = &'static Self::Step>;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    /// Returns the steps of the story.
    fn steps(&self) -> Self::StepIter;
}
