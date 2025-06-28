#[narrative::story("Story with missing step parameter")]
trait MissingParameterStory {
    #[step("When I perform an action")]
    fn perform_action(&self, action: &str); // Missing 'target' parameter
}

fn main() {}
