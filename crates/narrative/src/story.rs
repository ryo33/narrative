use std::future::Future;

use crate::{
    runner::{AsyncStoryRunner, DefaultStoryRunner, StoryRunner},
    step::{DynStep, Run, RunAsync, Step},
    value::{BoxedValue, Value},
};

/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext {
    type Step: Step + 'static;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    fn consts(
        &self,
    ) -> impl Iterator<Item = impl StoryConst + Send + Sync + 'static> + Send + Sync + 'static;
    /// Returns the steps of the story.
    fn steps(&self) -> impl Iterator<Item = Self::Step> + Send + Sync + 'static;
}

pub trait StoryConst: Clone + std::fmt::Debug {
    /// Returns the name of the constant value.
    fn name(&self) -> &'static str;
    /// Returns the type of the constant value.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the constant value.
    fn expr(&self) -> &'static str;
    /// Returns the value of the constant.
    fn value(&self) -> impl Value;
}

#[derive(Clone, Copy)]
pub struct DynStoryContext {
    story_title: &'static str,
    story_id: &'static str,
    consts: fn() -> Box<dyn Iterator<Item = DynStoryConst> + Send + Sync>,
    steps: fn() -> Box<dyn Iterator<Item = DynStep> + Send + Sync>,
}

impl DynStoryContext {
    pub const fn new(
        story_title: &'static str,
        story_id: &'static str,
        consts: fn() -> Box<dyn Iterator<Item = DynStoryConst> + Send + Sync>,
        steps: fn() -> Box<dyn Iterator<Item = DynStep> + Send + Sync>,
    ) -> Self {
        Self {
            story_title,
            story_id,
            consts,
            steps,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DynStoryConst {
    name: &'static str,
    ty: &'static str,
    expr: &'static str,
    value: fn() -> BoxedValue,
    obj_value: fn() -> BoxedValue,
}

impl DynStoryConst {
    pub const fn new(
        name: &'static str,
        ty: &'static str,
        expr: &'static str,
        value: fn() -> BoxedValue,
        obj_value: fn() -> BoxedValue,
    ) -> Self {
        Self {
            name,
            ty,
            expr,
            value,
            obj_value,
        }
    }
}

impl std::fmt::Debug for DynStoryConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.obj_value)().fmt(f)
    }
}

impl StoryContext for DynStoryContext {
    type Step = DynStep;

    fn story_title(&self) -> String {
        self.story_title.to_string()
    }

    fn story_id(&self) -> &'static str {
        self.story_id
    }

    fn consts(
        &self,
    ) -> impl Iterator<Item = impl StoryConst + Send + Sync + 'static> + Send + Sync + 'static {
        (self.consts)()
    }

    fn steps(&self) -> impl Iterator<Item = Self::Step> + Send + Sync + 'static {
        (self.steps)()
    }
}

impl StoryConst for DynStoryConst {
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

pub trait RunStory<T, S, E> {
    fn run_story(&self, env: &mut S) -> Result<(), E>;
    fn run_story_with_runner(&self, env: &mut S, runner: &mut impl StoryRunner<E>)
        -> Result<(), E>;
}

impl<T, S, E> RunStory<T, S, E> for T
where
    T: StoryContext + Copy,
    T::Step: Run<S, E>,
{
    fn run_story(&self, env: &mut S) -> Result<(), E> {
        let mut runner = DefaultStoryRunner;
        Self::run_story_with_runner(self, env, &mut runner)
    }
    fn run_story_with_runner(
        &self,
        env: &mut S,
        runner: &mut impl StoryRunner<E>,
    ) -> Result<(), E> {
        runner.start_story(*self)?;
        for step in self.steps() {
            runner.run_step(step, env)?;
        }
        runner.end_story(*self)?;
        Ok(())
    }
}

pub trait RunStoryAsync<T, S, E> {
    fn run_story_async(&self, env: &mut S) -> impl Future<Output = Result<(), E>> + Send;
    fn run_story_with_runner_async(
        &self,
        env: &mut S,
        runner: &mut (impl AsyncStoryRunner<E> + Send),
    ) -> impl Future<Output = Result<(), E>> + Send;
}

impl<T, S, E> RunStoryAsync<T, S, E> for T
where
    T: StoryContext + Copy + Send + Sync,
    T::Step: RunAsync<S, E> + Send,
    S: Send,
{
    async fn run_story_async(&self, env: &mut S) -> Result<(), E> {
        let mut runner = DefaultStoryRunner;
        Self::run_story_with_runner_async(self, env, &mut runner).await
    }
    async fn run_story_with_runner_async(
        &self,
        env: &mut S,
        runner: &mut (impl AsyncStoryRunner<E> + Send),
    ) -> Result<(), E> {
        runner.start_story(*self)?;
        for step in self.steps() {
            runner.run_step_async(step, env).await?;
        }
        runner.end_story(*self)?;
        Ok(())
    }
}
