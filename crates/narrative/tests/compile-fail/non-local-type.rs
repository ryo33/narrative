// This type is defined outside the story trait
struct NotLocalType;

#[narrative::story("Test non-local type error")]
trait TestStory {
    #[step("Given a non-local type")]
    fn use_non_local_type(&self, arg: NotLocalType);
}

fn main() {}