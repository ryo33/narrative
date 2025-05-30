#[narrative::story("Empty story is valid")]
trait EmptyStory {}

#[allow(dead_code)]
struct Env;

impl EmptyStory for Env {
    type Error = std::convert::Infallible;
}

#[test]
fn test_empty_story() {
    use narrative::story::RunStory as _;
    let mut env = Env;
    EmptyStoryContext.run_story(&mut env).unwrap();
}
