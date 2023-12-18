use std::io::{BufWriter, Write as _};

#[narrative::story("Say hello world")]
trait HelloWorld {
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

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
    let mut story = Env {
        buf: BufWriter::new(Vec::new()),
    };
    story.run_all().unwrap();
    assert_eq!(story.buf.into_inner().unwrap(), b"Hello, World!");
}
