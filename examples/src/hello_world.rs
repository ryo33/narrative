use std::io::{BufWriter, Write as _};

/// This is a hello world story.
#[narrative::story("Say hello world")]
trait HelloWorld {
    /// This is a step to say hello.
    #[step("Say hello")]
    fn say_hello();
    /// This is a step to say world.
    #[step("Say world")]
    fn say_world();
}

#[allow(dead_code)]
struct Env {
    buf: BufWriter<Vec<u8>>,
}

impl HelloWorld for Env {
    type Error = std::io::Error;

    fn say_hello(&mut self) -> Result<(), Self::Error> {
        write!(self.buf, "Hello, ")
    }

    fn say_world(&mut self) -> Result<(), Self::Error> {
        write!(self.buf, "World!")
    }
}

#[test]
fn test() {
    use narrative::story::RunStory as _;
    let mut env = Env {
        buf: BufWriter::new(Vec::new()),
    };
    HelloWorldContext.run_story(&mut env).unwrap();
    assert_eq!(env.buf.into_inner().unwrap(), b"Hello, World!");
}
