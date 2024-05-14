/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext: Sized {
    type Step: 'static;
    type Const: 'static;
    type StepIter: Iterator<Item = &'static Self::Step>;
    type ConstIter: Iterator<Item = &'static Self::Const>;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    fn consts(&self) -> Self::ConstIter;
    /// Returns the steps of the story.
    fn steps(&self) -> Self::StepIter;
}

pub trait StoryConst: Clone + std::fmt::Debug {
    /// Returns the name of the constant value.
    fn name(&self) -> &'static str;
    /// Returns the type of the constant value.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the constant value.
    fn expr(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Serializes the value to the given serializer.
    fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error>;
}
