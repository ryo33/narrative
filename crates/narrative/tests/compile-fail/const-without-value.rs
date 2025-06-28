#[narrative::story("Story with const missing value")]
trait ConstWithoutValueStory {
    const INVALID_CONST: i32; // Error: in a story, all consts must have a value

    #[step("When using const")]
    fn use_const(&self);
}

fn main() {}
