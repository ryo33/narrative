use narrative::{
    runner::{AsyncStoryRunner, StoryRunner},
    step::{Run, RunAsync, Step},
    story::{RunStory, RunStoryAsync, StoryContext},
};

#[allow(dead_code)]
struct LoggingStoryRunner {
    log: Vec<String>,
}

impl<E> StoryRunner<E> for LoggingStoryRunner {
    fn start_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.log
            .push(format!("Starting story: {}", story.story_title()));
        Ok(())
    }

    fn end_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.log
            .push(format!("Ending story: {}", story.story_title()));
        Ok(())
    }

    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: Step + Run<S, E>,
    {
        self.log.push(format!("Running step: {}", step.step_text()));
        step.run_with_runner(state, self)
    }

    fn run_nested_story<S, Env>(
        &mut self,
        step: impl Step,
        nested_story: S,
        env: &mut Env,
    ) -> Result<(), E>
    where
        S::Step: Run<Env, E>,
        S: StoryContext + RunStory<S, Env, E>,
    {
        self.log.push(format!(
            "Starting nested story: {}; {}",
            step.step_text(),
            nested_story.story_title(),
        ));
        nested_story.run_story_with_runner(env, self)
    }
}

impl<E> AsyncStoryRunner<E> for LoggingStoryRunner {
    fn start_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.log
            .push(format!("Starting story: {}", story.story_title()));
        Ok(())
    }

    fn end_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.log
            .push(format!("Ending story: {}", story.story_title()));
        Ok(())
    }

    fn run_step_async<T, S>(
        &mut self,
        step: T,
        state: &mut S,
    ) -> impl std::future::Future<Output = Result<(), E>> + Send
    where
        T: Step + RunAsync<S, E> + Send,
        S: Send,
    {
        // Async recursion happens here by passing `self`, so Box::pin is required.
        Box::pin(async move {
            self.log.push(format!("Running step: {}", step.step_text()));
            step.run_with_runner_async(state, self).await
        })
    }

    async fn run_nested_story_async<S, Env>(
        &mut self,
        step: impl Step + Send,
        nested_story: S,
        env: &mut Env,
    ) -> Result<(), E>
    where
        S: StoryContext + RunStoryAsync<S, Env, E> + Send,
        Env: Send,
        S::Step: RunAsync<Env, E> + Send,
    {
        self.log.push(format!(
            "Starting nested story: {}; {}",
            step.step_text(),
            nested_story.story_title(),
        ));
        nested_story.run_story_async(env).await
    }
}

#[test]
fn test() {
    use crate::my_first_story::MyFirstStoryContext;
    use narrative::{environment::DummyEnvironment, story::RunStory};
    use std::convert::Infallible;
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment::<Infallible>::default();
    MyFirstStoryContext
        .run_story_with_runner(&mut env, &mut runner)
        .unwrap();
    assert_eq!(
        runner.log,
        vec![
            "Starting story: My First Story",
            "Running step: Hi, I'm a user: Ryo",
            "Running step: I have an apple",
            "Running step: I have 2 orages",
            "Running step: I should have 3 fruits",
            "Ending story: My First Story",
        ]
    );
}

#[test]
fn test_async() {
    use crate::async_story::MyFirstStoryContext;
    use narrative::{environment::DummyEnvironment, story::RunStoryAsync};
    use std::convert::Infallible;
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment::<Infallible>::default();
    futures::executor::block_on(
        MyFirstStoryContext.run_story_with_runner_async(&mut env, &mut runner),
    )
    .unwrap();
    assert_eq!(
        runner.log,
        vec![
            "Starting story: My First Story",
            "Running step: Hi, I'm a user",
            "Running step: I have an apple",
            "Running step: I have 2 orages",
            "Running step: I should have 3 fruits",
            "Ending story: My First Story",
        ]
    );
}

#[test]
fn test_sub_story() {
    use crate::sub_story::MainStoryContext;
    use narrative::{environment::DummyEnvironment, story::RunStory};
    use pretty_assertions::assert_eq;
    use std::convert::Infallible;
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment::<Infallible>::default();
    MainStoryContext
        .run_story_with_runner(&mut env, &mut runner)
        .unwrap();
    assert_eq!(
        runner.log,
        vec![
            "Starting story: This is a main story",
            "Running step: do sub story",
            "Starting nested story: do sub story; This is a sub story",
            "Starting story: This is a sub story",
            "Running step: sub_step_1",
            "Running step: sub_step_2",
            "Ending story: This is a sub story",
            "Running step: do sub story with args",
            "Starting nested story: do sub story with args; This is a sub story 2",
            "Starting story: This is a sub story 2",
            "Running step: sub_step_3",
            "Running step: sub_step_4",
            "Ending story: This is a sub story 2",
            "Ending story: This is a main story",
        ]
    );
}
