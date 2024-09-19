use narrative::environment::DummyEnvironment;

use crate::run_test;

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
    run_test!(ComplexStepTypeContext, DummyEnvironment);
}
