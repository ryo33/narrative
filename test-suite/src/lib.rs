#![cfg(test)]
#![deny(clippy::all)]

use narrative::{runner::StoryRunner, step::Run};

mod multiple_stories_in_one_file;
mod step_arg;
mod story_consts;

#[derive(Default)]
pub struct TestRunner {
    story_queue_depth: usize,
}

impl<E: std::fmt::Display> StoryRunner<E> for TestRunner {
    fn start_story(&mut self, story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.story_queue_depth += 1;
        eprintln!(
            "{}story: {}",
            "  ".repeat(self.story_queue_depth),
            story.story_title()
        );
        Ok(())
    }

    fn end_story(&mut self, _story: impl narrative::story::StoryContext) -> Result<(), E> {
        self.story_queue_depth -= 1;
        Ok(())
    }

    fn run_step<T, S>(&mut self, step: T, state: &mut S) -> Result<(), E>
    where
        T: narrative::step::Step + narrative::step::Run<S, E>,
    {
        eprintln!(
            "{}step: {}",
            "  ".repeat(self.story_queue_depth + 1),
            step.step_text()
        );
        if let Err(err) = step.run(state) {
            panic!("step {} failed: {}", step.step_text(), err);
        }
        Ok(())
    }

    fn run_nested_story<S, Env>(
        &mut self,
        _step: impl narrative::step::Step,
        nested_story: S,
        env: &mut Env,
    ) -> Result<(), E>
    where
        S::Step: narrative::step::Run<Env, E>,
        S: narrative::story::StoryContext + narrative::story::RunStory<S, Env, E>,
    {
        eprintln!(
            "{}nested story: {}",
            "  ".repeat(self.story_queue_depth + 1),
            nested_story.story_title()
        );
        // Test that we can run a nested story step by step by using the provided bound.
        if false {
            for step in nested_story.steps() {
                step.run_with_runner(env, self)?;
            }
        }
        nested_story.run_story_with_runner(env, self)
    }
}
