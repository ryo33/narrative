#[narrative::story("Consts")]
trait Consts {
    const NAME: &'static str = "Ryo";
    const ID: &'static str = "ryo33";
}

#[test]
fn accessible_through_context() {
    assert_eq!(ConstsContext::ID, "ryo33");
    assert_eq!(ConstsContext::NAME, "Ryo");
}
