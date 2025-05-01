use crate::{
    runner::{AsyncStoryRunner, StoryRunner},
    story::{BoxedStoryContext, StoryContext},
};

// T and E can be any type by implementing the story trait in any way.
pub trait Step {
    /// Returns the text representation of the step.
    fn step_text(&self) -> String;
    /// Returns the id, which is the method name, of the step.
    fn step_id(&self) -> &'static str;
    /// Returns the arguments of the step.
    fn args(&self) -> impl Iterator<Item = impl StepArg + 'static> + 'static;
    /// Returns the parent story of the step.
    fn story(&self) -> impl StoryContext<Step = Self> + 'static;
    /// Returns the sub story that this step references.
    fn nested_story(&self) -> Option<impl StoryContext + 'static>;
}

pub trait Run<T, E>: Step {
    /// Runs the step.
    fn run(&self, story: &mut T) -> Result<(), E>;
    /// Runs the step, but with a runner if the step has a sub story.
    fn run_with_runner(&self, story: &mut T, runner: &mut impl StoryRunner<E>) -> Result<(), E>;
}

pub trait RunAsync<T, E>: Step {
    /// Runs the step asynchronously.
    fn run_async(&self, story: &mut T) -> impl std::future::Future<Output = Result<(), E>> + Send;
    /// Runs the step asynchronously, but with a runner if the step has a sub story.
    fn run_with_runner_async(
        &self,
        story: &mut T,
        runner: &mut (impl AsyncStoryRunner<E> + Send),
    ) -> impl std::future::Future<Output = Result<(), E>> + Send;
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
    /// Returns the value for serialization.
    fn serialize_value(&self) -> impl serde::Serialize + 'static;
    // TODO: fn schema() -> Schema;
}

mod private {
    pub trait SealedDynStep {}
    pub trait SealedDynStepArg {}
}

/// A trait object for [Step].
pub trait DynStep: private::SealedDynStep {
    /// Returns the text representation of the step.
    fn step_text(&self) -> String;
    /// Returns the id, which is the method name, of the step.
    fn step_id(&self) -> &'static str;
    /// Returns the arguments of the step.
    fn args(&self) -> Box<dyn Iterator<Item = Box<dyn DynStepArg>>>;
    /// Returns the sub story that this step references.
    fn nested_story(&self) -> Option<BoxedStoryContext>;
}

/// A trait object for [StepArg].
pub trait DynStepArg: private::SealedDynStepArg {
    /// Returns the name of the argument.
    fn name(&self) -> &'static str;
    /// Returns the type of the argument.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the argument.
    fn expr(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Returns the value for serialization.
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

    fn nested_story(&self) -> Option<BoxedStoryContext> {
        Step::nested_story(self).map(|s| Box::new(s) as BoxedStoryContext)
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
