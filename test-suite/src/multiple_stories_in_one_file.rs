use std::convert::Infallible;

#[narrative::story("Say hello world")]
trait HelloWorld {
    const A: &'static str = "a";
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

#[narrative::story("Say hello world 2")]
trait HelloWorld2 {
    const A: &'static str = "a";
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

#[narrative::story("Say hello world 3")]
trait HelloWorld3 {
    const A: &'static str = "a";
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

impl AsyncHelloWorld for Env {
    type Error = Infallible;

    async fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    async fn say_world(&mut self) -> Result<(), Self::Error> {
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

impl AsyncHelloWorld2 for Env {
    type Error = Infallible;

    async fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    async fn say_world(&mut self) -> Result<(), Self::Error> {
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

impl AsyncHelloWorld3 for Env {
    type Error = Infallible;

    async fn say_hello(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    async fn say_world(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
