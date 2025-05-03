use crate::{
    runner::{AsyncStoryRunner, StoryRunner},
    story::{DynStoryContext, StoryContext},
    value::{BoxedValue, Value},
};

// T and E can be any type by implementing the story trait in any way.
pub trait Step {
    /// Returns the text representation of the step.
    fn step_text(&self) -> String;
    /// Returns the id, which is the method name, of the step.
    fn step_id(&self) -> &'static str;
    /// Returns the arguments of the step.
    fn args(
        &self,
    ) -> impl Iterator<Item = impl StepArg + Send + Sync + 'static> + Send + Sync + 'static;
    /// Returns the parent story of the step.
    fn story(&self) -> impl StoryContext<Step = Self> + Send + Sync + 'static;
    /// Returns the sub story that this step references.
    fn nested_story(&self) -> Option<impl StoryContext + Send + Sync + 'static>;
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
    /// Returns the actual value of the argument.
    fn value(&self) -> impl Value;
    // TODO: fn schema() -> Schema;
}

#[derive(Clone, Copy)]
pub struct DynStep {
    step_text: fn() -> String,
    step_id: &'static str,
    args: fn() -> Box<dyn Iterator<Item = DynStepArg> + Send + Sync>,
    story: fn() -> DynStoryContext,
    nested_story: fn() -> Option<DynStoryContext>,
}

impl DynStep {
    pub const fn new(
        step_text: fn() -> String,
        step_id: &'static str,
        args: fn() -> Box<dyn Iterator<Item = DynStepArg> + Send + Sync>,
        story: fn() -> DynStoryContext,
        nested_story: fn() -> Option<DynStoryContext>,
    ) -> Self {
        Self {
            step_text,
            step_id,
            args,
            story,
            nested_story,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DynStepArg {
    name: &'static str,
    ty: &'static str,
    expr: &'static str,
    value: fn() -> BoxedValue,
    step_value: fn() -> BoxedValue,
}

impl DynStepArg {
    pub const fn new(
        name: &'static str,
        ty: &'static str,
        expr: &'static str,
        value: fn() -> BoxedValue,
        step_value: fn() -> BoxedValue,
    ) -> Self {
        Self {
            name,
            ty,
            expr,
            value,
            step_value,
        }
    }
}

impl std::fmt::Debug for DynStepArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.step_value)().fmt(f)
    }
}

impl Step for DynStep {
    fn step_text(&self) -> String {
        (self.step_text)()
    }

    fn step_id(&self) -> &'static str {
        self.step_id
    }

    fn args(
        &self,
    ) -> impl Iterator<Item = impl crate::step::StepArg + Send + Sync + 'static> + Send + Sync + 'static
    {
        (self.args)()
    }

    fn story(&self) -> impl StoryContext<Step = Self> + Send + Sync + 'static {
        (self.story)()
    }

    fn nested_story(&self) -> Option<impl StoryContext + Send + Sync + 'static> {
        (self.nested_story)()
    }
}

impl StepArg for DynStepArg {
    fn name(&self) -> &'static str {
        self.name
    }

    fn ty(&self) -> &'static str {
        self.ty
    }

    fn expr(&self) -> &'static str {
        self.expr
    }

    fn value(&self) -> impl Value {
        (self.value)()
    }
}
