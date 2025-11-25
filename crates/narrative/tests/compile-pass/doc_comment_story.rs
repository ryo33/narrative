/// This is a my first story.
#[narrative::story("My First Story")]
trait MyFirstStory {
    /// This is a name.
    const NAME: &str = "Ryo";
    /// This is a step to say hello to the user.
    #[step("Hi, I'm a user: {NAME}")]
    fn as_a_user();
    /// This is a step to have one apple.
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    /// This is a step to have two oranges.
    #[step("I have {count} orages", count = 2)]
    fn have_two_oranges(count: u32);
    /// This is a step to should have three fruits.
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}

fn main() {}
