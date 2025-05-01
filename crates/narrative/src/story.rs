use std::future::Future;

use crate::{
    runner::{AsyncStoryRunner, DefaultStoryRunner, StoryRunner},
    step::{DynStep, Run, RunAsync, Step},
};

/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext: Sized {
    type Step: Step + 'static;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    fn consts(&self) -> impl Iterator<Item = impl StoryConst + Send + 'static> + Send + 'static;
    /// Returns the steps of the story.
    fn steps(&self) -> impl Iterator<Item = Self::Step> + Send + 'static;
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
    /// Returns the value for serialization.
    fn serialize_value(&self) -> impl serde::Serialize + 'static;
}

/// A boxed trait object for [StoryContext].
pub type BoxedStoryContext = Box<dyn DynStoryContext>;

mod private {
    pub trait SealedDynStoryContext {}
    pub trait SealedDynStoryConst {}
}

/// A trait object for [StoryContext].
pub trait DynStoryContext: private::SealedDynStoryContext {
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    /// Returns the constants of the story.
    fn consts(&self) -> Box<dyn Iterator<Item = Box<dyn DynStoryConst>>>;
    /// Returns the steps of the story.
    fn steps(&self) -> Box<dyn Iterator<Item = Box<dyn DynStep>>>;
}

/// A trait object for [StoryConst].
pub trait DynStoryConst: private::SealedDynStoryConst {
    /// Returns the name of the constant value.
    fn name(&self) -> &'static str;
    /// Returns the type of the constant value.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the constant value.
    fn expr(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Returns the value for serialization.
    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize>;
}

impl<T: StoryContext> private::SealedDynStoryContext for T {}
impl<T: StoryConst> private::SealedDynStoryConst for T {}

impl<T: StoryContext + private::SealedDynStoryContext> DynStoryContext for T {
    fn story_title(&self) -> String {
        StoryContext::story_title(self)
    }

    fn story_id(&self) -> &'static str {
        StoryContext::story_id(self)
    }

    fn consts(&self) -> Box<dyn Iterator<Item = Box<dyn DynStoryConst>>> {
        Box::new(StoryContext::consts(self).map(|c| Box::new(c) as Box<dyn DynStoryConst>))
    }

    fn steps(&self) -> Box<dyn Iterator<Item = Box<dyn DynStep>>> {
        Box::new(StoryContext::steps(self).map(|s| Box::new(s) as Box<dyn DynStep>))
    }
}

impl<T: StoryConst + private::SealedDynStoryConst> DynStoryConst for T {
    fn name(&self) -> &'static str {
        StoryConst::name(self)
    }

    fn ty(&self) -> &'static str {
        StoryConst::ty(self)
    }

    fn expr(&self) -> &'static str {
        StoryConst::expr(self)
    }

    fn debug_value(&self) -> String {
        StoryConst::debug_value(self)
    }

    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize> {
        Box::new(StoryConst::serialize_value(self))
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
