use std::convert::Infallible;

#[narrative::story("My First Story")]
trait MyFirstStory {
    const NAME: &'static str = "Ryo";
    #[step("Hi, I'm a user: {NAME}")]
    fn as_a_user();
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    #[step("I have {count} orages", count = 2)]
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
    MyFirstStoryEnv { sum: 0 }.run_all().unwrap();
}

#[test]
fn test_context() {
    assert_eq!(MyFirstStoryContext::NAME, "Ryo");
}
