use std::future::Future;

use crate::{
    step::{Run, RunAsync, Step},
    story::{RunStory, RunStoryAsync, StoryContext},
};

/// A trait for running a story.
pub trait StoryRunner<E> {
    /// Called when the root or a nested story starts.
    fn start_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    /// Called when the root or a nested story ends.
    fn end_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    /// Executes a step.
    /// If you call `step.run_with_runner(env, self)`, the runner will be passed to the nested story if it exists.
    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + Run<S, E>;
    /// Executes a nested story referenced by a parent step.
    /// You can run a nested story step by step with the `Run` trait or all at once with the `RunStory` trait.
    /// If you call those methods with `_with_runner` variants with `self`, the runner will be passed to the nested story.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn run_nested_story<S, Env>(
    ///     &mut self,
    ///     _step: impl narrative::step::Step,
    ///     nested_story: S,
    ///     env: &mut Env,
    /// ) -> Result<(), E>
    /// where
    ///     S::Step: narrative::step::Run<Env, E>,
    ///     S: narrative::story::StoryContext + narrative::story::RunStory<S, Env, E>,
    /// {
    ///     if self.run_nested_story_step_by_step {
    ///         for step in nested_story.steps() {
    ///             step.run_with_runner(env, self)?;
    ///         }
    ///     } else {
    ///         nested_story.run_story_with_runner(env, self)
    ///     }
    /// }
    /// ```
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

/// A trait for running a story asynchronously.
pub trait AsyncStoryRunner<E> {
    /// Called when the root or a nested story starts.
    fn start_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    /// Called when the root or a nested story ends.
    fn end_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    /// Executes a step asynchronously.
    /// If you call `step.run_with_runner_async(env, self)`, the runner will be passed to the nested story if it exists.
    fn run_step_async<T, Env>(
        &mut self,
        step: T,
        state: &mut Env,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        T: Step + RunAsync<Env, E> + Send + Sync,
        Env: Send;
    /// Executes a nested story referenced by a parent step asynchronously.
    /// See [StoryRunner::run_nested_story] for more details.
    fn run_nested_story_async<S, Env>(
        &mut self,
        step: impl Step + Send,
        nested_story: S,
        env: &mut Env,
    ) -> impl Future<Output = Result<(), E>> + Send
    where
        S: StoryContext + RunStoryAsync<S, Env, E> + Send + Sync,
        Env: Send,
        S::Step: RunAsync<Env, E> + Send + Sync;
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
    async fn run_step_async<T, Env>(&mut self, step: T, state: &mut Env) -> Result<(), E>
    where
        T: Step + RunAsync<Env, E> + Send + Sync,
        Env: Send,
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
        S: StoryContext + RunStoryAsync<S, Env, E> + Send + Sync,
        Env: Send,
        S::Step: RunAsync<Env, E> + Send + Sync,
    {
        nested_story.run_story_async(env).await
    }
}
