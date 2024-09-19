#![cfg(test)]
#![deny(clippy::all)]

mod step_arg;
mod story_consts;

macro_rules! run_test {
    ($context:expr, $env:expr) => {
        use narrative::step::Step as _;
        use narrative::story::StoryContext as _;
        let context = $context;
        eprintln!("story: {}", context.story_title());
        for step in context.steps() {
            eprintln!("  step: {}", step.step_text());
            if let Err(err) = step.run(&mut $env) {
                panic!("step {} failed: {}", step.step_text(), err);
            }
        }
    };
}

use run_test;
