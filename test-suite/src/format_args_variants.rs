use narrative::step::{Step, StepArg};

#[narrative::story("Format args variants")]
trait FormatArgsVariants {
    const NAME: &'static str = "ryo";
    const UNUSED: &'static str = "unused";
    #[step("Format args variants", text = format!("{}", "hello"))]
    fn variant_1(text: String);
    #[step("Format args variants", text = format!("{UNUSED}", UNUSED = NAME))]
    fn variant_2(text: String);
    #[step("Format args variants", text = format!("{NAME}"))]
    fn variant_3(text: String);
    #[step("Format args variants", text = format!("{NAME}", NAME = NAME))]
    fn variant_4(text: String);
    #[step("Format args variants {NAME}, {UNUSED}", UNUSED = "aa")]
    fn variant_5(UNUSED: &str);
}

#[test]
fn test_format_args_variants() {
    let steps = FormatArgsVariantsContext.steps().collect::<Vec<_>>();
    assert_eq!(steps.len(), 5);
    assert_eq!(steps[0].args().next().unwrap().debug_value(), r#""hello""#);
    assert_eq!(steps[1].args().next().unwrap().debug_value(), r#""ryo""#);
    assert_eq!(steps[2].args().next().unwrap().debug_value(), r#""ryo""#);
    assert_eq!(steps[3].args().next().unwrap().debug_value(), r#""ryo""#);
    assert_eq!(steps[4].step_text(), r#"Format args variants ryo, aa"#);
    assert_eq!(steps[4].args().next().unwrap().debug_value(), r#""aa""#);
}
