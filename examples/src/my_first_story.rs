use std::convert::Infallible;

#[narrative::story("My First Story")]
trait MyFirstStory {
    const NAME: &str = "Ryo";
    #[step("Hi, I'm a user: {NAME}")]
    fn as_a_user();
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    #[step("I have {count} oranges", count = 2)]
    fn have_two_oranges(count: u32);
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}

#[allow(dead_code)]
struct MyFirstStoryEnv {
    sum: u32,
}

impl MyFirstStory for MyFirstStoryEnv {
    type Error = Infallible;

    fn as_a_user(&mut self) -> Result<(), Self::Error> {
        println!("Hi, I'm a user: {}", Self::NAME);
        Ok(())
    }

    fn have_one_apple(&mut self, count: u32) -> Result<(), Self::Error> {
        self.sum += count;
        Ok(())
    }

    fn have_two_oranges(&mut self, count: u32) -> Result<(), Self::Error> {
        self.sum += count;
        Ok(())
    }

    fn should_have_three_fruits(&mut self, total: u32) -> Result<(), Self::Error> {
        assert_eq!(self.sum, total);
        Ok(())
    }
}

#[test]
fn test() {
    use narrative::story::RunStory as _;
    let mut env = MyFirstStoryEnv { sum: 0 };
    MyFirstStoryContext.run_story(&mut env).unwrap();
}

#[test]
fn test_context() {
    use narrative::prelude::*;
    assert_eq!(MyFirstStoryContext::NAME, "Ryo");
    let consts = MyFirstStoryContext.consts().collect::<Vec<_>>();
    assert_eq!(consts.len(), 1);
    assert_eq!(format!("{:?}", consts[0]), "NAME: &str = \"Ryo\"");
    let steps = MyFirstStoryContext.steps().collect::<Vec<_>>();
    assert_eq!(steps.len(), 4);
    assert_eq!(steps[0].args().count(), 0);
    assert_eq!(steps[0].step_text(), "Hi, I'm a user: Ryo");
    let args = steps[1].args().collect::<Vec<_>>();
    assert_eq!(args.len(), 1);
    assert_eq!(format!("{:?}", args[0]), "count: u32 = 1");
    assert_eq!(steps[1].step_text(), "I have an apple");
    assert_eq!(steps[2].step_text(), "I have 2 oranges");
    assert_eq!(steps[3].step_text(), "I should have 3 fruits");
}
