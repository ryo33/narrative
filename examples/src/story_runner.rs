use narrative::{
    runner::{AsyncStoryRunner, StoryRunner},
    step::{Run, RunAsync, Step},
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
}

#[test]
fn test() {
    use crate::my_first_story::MyFirstStoryContext;
    use narrative::{environment::DummyEnvironment, story::RunStory};
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment;
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
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment;
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
    let mut runner = LoggingStoryRunner { log: vec![] };
    let mut env = DummyEnvironment;
    MainStoryContext
        .run_story_with_runner(&mut env, &mut runner)
        .unwrap();
    assert_eq!(
        runner.log,
        vec![
            "Starting story: This is a main story",
            "Running step: do sub story",
            "Starting story: This is a sub story",
            "Running step: sub_step_1",
            "Running step: sub_step_2",
            "Ending story: This is a sub story",
            "Running step: do sub story with args",
            "Starting story: This is a sub story",
            "Running step: sub_step_1",
            "Running step: sub_step_2",
            "Ending story: This is a sub story",
            "Ending story: This is a main story",
        ]
    );
}
