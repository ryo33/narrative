use crate::story::{BoxedStoryContext, StoryContext};

// T and E can be any type by implementing the story trait in any way.
pub trait Step {
    fn step_text(&self) -> String;
    fn step_id(&self) -> &'static str;
    fn args(&self) -> impl Iterator<Item = impl StepArg + 'static> + 'static;
    fn story(&self) -> Option<impl StoryContext + 'static>;
}

pub trait Run<T, E>: Step {
    fn run(&self, story: &mut T) -> Result<(), E>;
}

pub trait RunAsync<T, E>: Step {
    fn run_async(&self, story: &mut T) -> impl std::future::Future<Output = Result<(), E>> + Send;
}

pub trait StepArg: Clone + std::fmt::Debug {
    /// Returns the name of the argument.
    fn name(&self) -> &'static str;
    /// Returns the type of the argument.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the argument.
    fn expr(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Serializes the value to the given serializer.
    fn serialize_value(&self) -> impl serde::Serialize + 'static;
    // TODO: fn schema() -> Schema;
}

mod private {
    pub trait SealedDynStep {}
    pub trait SealedDynStepArg {}
}

pub trait DynStep: private::SealedDynStep {
    fn step_text(&self) -> String;
    fn step_id(&self) -> &'static str;
    fn args(&self) -> Box<dyn Iterator<Item = Box<dyn DynStepArg>>>;
    fn story(&self) -> Option<BoxedStoryContext>;
}

pub trait DynStepArg: private::SealedDynStepArg {
    fn name(&self) -> &'static str;
    fn ty(&self) -> &'static str;
    fn expr(&self) -> &'static str;
    fn debug_value(&self) -> String;
    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize>;
}

impl<T: Step> private::SealedDynStep for T {}
impl<T: StepArg> private::SealedDynStepArg for T {}

impl<T: Step + private::SealedDynStep> DynStep for T {
    fn step_text(&self) -> String {
        Step::step_text(self)
    }

    fn step_id(&self) -> &'static str {
        Step::step_id(self)
    }

    fn args(&self) -> Box<dyn Iterator<Item = Box<dyn DynStepArg>>> {
        Box::new(Step::args(self).map(|a| Box::new(a) as Box<dyn DynStepArg>))
    }

    fn story(&self) -> Option<BoxedStoryContext> {
        Step::story(self).map(|s| Box::new(s) as BoxedStoryContext)
    }
}

impl<T: StepArg + private::SealedDynStepArg> DynStepArg for T {
    fn name(&self) -> &'static str {
        StepArg::name(self)
    }

    fn ty(&self) -> &'static str {
        StepArg::ty(self)
    }

    fn expr(&self) -> &'static str {
        StepArg::expr(self)
    }

    fn debug_value(&self) -> String {
        StepArg::debug_value(self)
    }

    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize> {
        Box::new(StepArg::serialize_value(self))
    }
}
