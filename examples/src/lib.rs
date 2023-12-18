use std::convert::Infallible;

#[narrative::story("My First Story")]
trait MyFirstStory {
    #[step("Hi, I'm a user")]
    fn as_a_user();
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    #[step("I have {count} orages", count = 2)]
    fn have_two_oranges(count: u32);
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}

struct MyFirstStoryEnv {
    sum: u32,
}

impl MyFirstStory for MyFirstStoryEnv {
    type Error = Infallible;

    fn as_a_user(&mut self) -> Result<(), Self::Error> {
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
    let mut story = MyFirstStoryEnv { sum: 0 };
    story.get_context().steps().for_each(|step| {
        step.run(&mut story).unwrap();
    });
    MyFirstStoryEnv::context();
}
