// This type is defined outside the story trait
// but does not implement StoryOwnedType or TestStoryLocalType
#[derive(Debug, Clone, serde::Serialize)]
struct NotLocalType;

#[narrative::story("Test non-local type error")]
trait TestStory {
    #[step("Given a non-local type", arg = NotLocalType)]
    fn use_non_local_type(&self, arg: NotLocalType);
}

fn main() {}
