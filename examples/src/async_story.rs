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

impl AsyncMyFirstStory for MyFirstStoryEnv {
    type Error = Infallible;

    fn as_a_user(&mut self) -> narrative::BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn have_one_apple(&mut self, count: u32) -> narrative::BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            self.sum += count;
            Ok(())
        })
    }

    fn have_two_oranges(
        &mut self,
        count: u32,
    ) -> narrative::BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            self.sum += count;
            Ok(())
        })
    }

    fn should_have_three_fruits(
        &mut self,
        total: u32,
    ) -> narrative::BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            assert_eq!(self.sum, total);
            Ok(())
        })
    }
}

#[test]
fn test() {
    let mut env = MyFirstStoryEnv { sum: 0 };
    let _ = futures::executor::block_on(env.run_all_async());
}
