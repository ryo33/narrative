mod sub_story {
    #[narrative::story("Sub story from other module")]
    trait SubStory {
        #[step("Sub step 1")]
        fn sub_step_1();
    }
}

use narrative::step::Step;
use sub_story::*;

#[narrative::story("Main story")]
trait MainStory {
    #[step(story: SubStory, "Run sub story")]
    fn main_step_1();
}

#[test]
fn test_sub_story_from_other_module() {
    let steps = MainStoryContext.steps().collect::<Vec<_>>();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].step_text(), "Run sub story");
}
