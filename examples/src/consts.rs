#[narrative::story("Consts")]
trait Consts {
    const NAME: &'static str = "Ryo";
    const ID: &'static str = "ryo33";
    #[step("dummy", name = NAME)]
    fn dummy_step(name: &str);

    #[step("use const in format!", url = format!("https://example.com/{ID}"))]
    fn format_step(url: String);

    #[step("use const in step text name: {NAME}")]
    fn format_step_in_step_text();
}

#[test]
fn accessible_through_context() {
    use narrative::step::Step;
    use narrative::step::StepArg;
    use serde_json::json;
    assert_eq!(ConstsContext::ID, "ryo33");
    assert_eq!(ConstsContext::NAME, "Ryo");

    let steps = ConstsContext.steps().collect::<Vec<_>>();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].args().count(), 1);
    let arg = steps[0].args().next().unwrap();
    assert_eq!(arg.debug_value(), r#""Ryo""#);
    assert_eq!(
        serde_json::to_value(arg.serialize_value()).unwrap(),
        json!("Ryo")
    );

    assert_eq!(steps[1].args().count(), 1);
    let arg = steps[1].args().next().unwrap();
    assert_eq!(arg.debug_value(), r#""https://example.com/ryo33""#);
    assert_eq!(
        serde_json::to_value(arg.serialize_value()).unwrap(),
        json!("https://example.com/ryo33")
    );

    assert_eq!(steps[2].args().count(), 0);
    assert_eq!(steps[2].step_text(), "use const in step text name: Ryo");
}
