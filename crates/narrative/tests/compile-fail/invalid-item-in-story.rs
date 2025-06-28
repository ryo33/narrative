#[narrative::story("Story with invalid item")]
trait InvalidItemStory {
    // Type aliases are not allowed in stories
    type SomeType = String;

    #[step("When using type")]
    fn use_type(&self);
}

fn main() {}
