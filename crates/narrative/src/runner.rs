use std::future::Future;

use crate::{
    step::{Run, RunAsync, Step},
    story::StoryContext,
};

pub trait StoryRunner<E> {
    fn start_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn end_story(&mut self, story: impl StoryContext) -> Result<(), E>;
    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + Run<S, E>;
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
}

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
}
