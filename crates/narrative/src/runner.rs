use std::future::Future;

use crate::{
    step::{Run, RunAsync, Step},
    story::{RunStory, RunStoryAsync, StoryContext},
};

pub trait StoryRunner<E> {
    fn start_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn end_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + Run<S, E>;
    /// Executes a nested story referenced by a parent step.
    fn run_nested_story<S, Env>(
        &mut self,
        step: impl Step,
        nested_story: S,
        env: &mut Env,
    ) -> Result<(), E>
    where
        S::Step: Run<Env, E>,
        S: StoryContext + RunStory<S, Env, E>;
}

pub trait AsyncStoryRunner<E> {
    fn start_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn end_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn run_step_async<T, S>(
        &mut self,
        step: T,
        state: &mut S,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        T: Step + RunAsync<S, E> + Send,
        S: Send;
    /// Executes a nested story referenced by a parent step asynchronously.
    fn run_nested_story_async<S, Env>(
        &mut self,
        step: impl Step + Send,
        nested_story: S,
        env: &mut Env,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        S: StoryContext + RunStoryAsync<S, Env, E> + Send,
        Env: Send,
        S::Step: RunAsync<Env, E> + Send;
}

/// The default story runner that executes steps sequentially without extra logic.
pub struct DefaultStoryRunner;

impl<E> StoryRunner<E> for DefaultStoryRunner {
    #[inline]
    fn start_story(&mut self, _story: impl StoryContext) -> Result<(), E> {
        Ok(())
    }

    #[inline]
    fn end_story(&mut self, _story: impl StoryContext) -> Result<(), E> {
        Ok(())
    }

    #[inline]
    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + Run<S, E>,
    {
        step.run(state)
    }

    #[inline]
    fn run_nested_story<NestedCtx: StoryContext + RunStory<NestedCtx, NestedImpl, E>, NestedImpl>(
        &mut self,
        _parent_step: impl Step,
        nested_context: NestedCtx,
        nested_impl: &mut NestedImpl,
    ) -> Result<(), E>
    where
        NestedCtx::Step: Run<NestedImpl, E>,
    {
        nested_context.run_story(nested_impl)
    }
}

impl<E> AsyncStoryRunner<E> for DefaultStoryRunner {
    #[inline]
    fn start_story(&mut self, _story: impl StoryContext) -> Result<(), E> {
        Ok(())
    }

    #[inline]
    fn end_story(&mut self, _story: impl StoryContext) -> Result<(), E> {
        Ok(())
    }

    #[inline]
    async fn run_step_async<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + RunAsync<S, E> + Send,
        S: Send,
    {
        step.run_async(state).await
    }

    #[inline]
    async fn run_nested_story_async<S, Env>(
        &mut self,
        _step: impl Step + Send,
        nested_story: S,
        env: &mut Env,
    ) -> Result<(), E>
    where
        S: StoryContext + RunStoryAsync<S, Env, E> + Send,
        Env: Send,
        S::Step: RunAsync<Env, E> + Send,
    {
        nested_story.run_story_async(env).await
    }
}
