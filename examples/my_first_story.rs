#[narrative::story(with = {
    struct User {
        name: String,
    }
})]
trait MyFirstStory {
    /// Hi, I'm a user
    fn as_a_user();
    /// I have an apple
    fn have_one_apple(#[given(1)] count: u32);
    /// I have {count} orages
    fn have_two_oranges(#[given(2)] count: u32);
    /// I should have {total} fruits
    fn should_have_three_fruits(#[given(3)] total: u32);
}

#[narrative::story]
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

struct A;

impl std::future::Future for A {}

fn main() {}
