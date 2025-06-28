#[narrative::story("Story with unbound parameter")]
trait UnboundParameterStory {
    #[step("When I do something", unused_param = "unused")]
    fn do_something(&self); // unused_param is not used
}

fn main() {}
