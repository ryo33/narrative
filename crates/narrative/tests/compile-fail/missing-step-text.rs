#[narrative::story("Story with invalid step")]
trait InvalidStepStory {
    #[step] // Missing required text parameter
    fn invalid_step(&self);
}

fn main() {}
