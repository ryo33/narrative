use std::convert::Infallible;

use narrative::environment::DummyEnvironment;

#[narrative::story("Say hello world")]
trait SubHelloWorld {
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
}

#[narrative::story("Say hello world")]
trait HelloWorld {
    const A: &str = "a";
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
    #[step(story: SubHelloWorld, "Say hello world in sub story")]
    fn say_hello_world();
}

#[narrative::story("Say hello world 2")]
trait HelloWorld2 {
    const A: &str = "a";
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
    #[step(story: SubHelloWorld, "Say hello world in sub story")]
    fn say_hello_world();
}

#[narrative::story("Say hello world 3")]
trait HelloWorld3 {
    const A: &str = "a";
    #[step("Say hello")]
    fn say_hello();
    #[step("Say world")]
    fn say_world();
    #[step(story: SubHelloWorld, "Say hello world in sub story")]
    fn say_hello_world();
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

    fn say_hello_world(&mut self) -> Result<impl SubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
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

    fn say_hello_world(
        &mut self,
    ) -> Result<impl AsyncSubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
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

    fn say_hello_world(&mut self) -> Result<impl SubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
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

    fn say_hello_world(
        &mut self,
    ) -> Result<impl AsyncSubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
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

    fn say_hello_world(&mut self) -> Result<impl SubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
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

    fn say_hello_world(
        &mut self,
    ) -> Result<impl AsyncSubHelloWorld<Error = Self::Error>, Self::Error> {
        Ok(DummyEnvironment::<Self::Error>::default())
    }
}
