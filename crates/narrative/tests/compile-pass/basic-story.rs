#[narrative::story("My First Story")]
trait MyFirstStory {
    const NAME: &str = "Ryo";
    #[step("Hi, I'm a user: {NAME}")]
    fn as_a_user();
    #[step("I have an apple", count = 1)]
    fn have_one_apple(count: u32);
    #[step("I have {count} orages", count = 2)]
    fn have_two_oranges(count: u32);
    #[step("I should have {total} fruits", total = 3)]
    fn should_have_three_fruits(total: u32);
}

fn main() {}
