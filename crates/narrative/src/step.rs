// T and E can be any type by implementing the story trait in any way.
pub trait Step {
    type Id: StepId;
    type Arg: StepArg + 'static;
    type ArgIter: Iterator<Item = &'static Self::Arg>;
    fn step_text(&self) -> String;
    fn args(&self) -> Self::ArgIter;
    fn id(&self) -> Self::Id;
}

pub trait Run<T, E> {
    fn run(&self, story: &mut T) -> Result<(), E>;
}

#[async_trait::async_trait]
pub trait RunAsync<T, E>: Step {
    async fn run_async(&self, story: &mut T) -> Result<(), E>;
}

pub trait StepId: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + 'static {
    fn index(&self) -> usize;
}

pub trait StepArg: Clone + std::fmt::Debug {
    /// Returns the name of the argument.
    fn name(&self) -> &'static str;
    /// Returns the type of the argument.
    fn ty(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Serializes the value to the given serializer.
    fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error>;
    // TODO: fn schema() -> Schema;
}
