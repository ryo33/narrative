mod literal_types;

#[narrative::story("Story consts")]
trait StoryConsts {
    const NUMBER: u32 = 42;

    #[step("Step")]
    fn step(&self);
}
