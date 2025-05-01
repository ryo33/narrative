use std::convert::Infallible;

#[narrative::story("Say hello world")]
trait HelloWorld {
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

#[narrative::story("Say hello world 2")]
trait HelloWorld2 {
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

#[narrative::story("Say hello world 3")]
trait HelloWorld3 {
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

pub struct Env;

impl HelloWorld for Env {
    type Error = Infallible;

    fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn say_world(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl HelloWorld2 for Env {
    type Error = Infallible;

    fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn say_world(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl HelloWorld3 for Env {
    type Error = Infallible;

    fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn say_world(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
