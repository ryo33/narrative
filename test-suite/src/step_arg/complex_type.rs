use std::convert::Infallible;

use narrative::{environment::DummyEnvironment, story::RunStory};

use crate::TestRunner;

#[narrative::story("Complex step type")]
trait ComplexStepType {
    #[step("Complex step", arg = &[
		&["a", "b"],
		&["c", "d", "e"],
	])]
    fn complex_step(&self, arg: &[&[&str]]);
}

#[test]
fn test_complex_step_type() {
    let mut env = DummyEnvironment::<Infallible>::default();
    let mut runner = TestRunner::default();
    ComplexStepTypeContext
        .run_story_with_runner(&mut env, &mut runner)
        .unwrap();
}
