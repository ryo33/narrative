use std::convert::Infallible;

#[narrative::story("This is a sub story")]
trait SubStory {
    #[step("sub_step_1")]
    fn sub_step_1();
    #[step("sub_step_2")]
    fn sub_step_2();
}

#[narrative::story("This is a sub story 2")]
trait SubStory2 {
    #[step("sub_step_3")]
    fn sub_step_3();
    #[step("sub_step_4")]
    fn sub_step_4();
}

#[narrative::story("This is a main story")]
trait MainStory {
    #[step(story: SubStory, "do sub story")]
    fn main_step_1();
    #[step(story: SubStory2, "do sub story with args", arg = 2)]
    fn main_step_2(arg: i32);
}

#[allow(dead_code)]
struct SubStoryImpl<'a> {
    state: &'a mut Vec<String>,
    arg: Option<i32>,
}

impl AsyncSubStory for SubStoryImpl<'_> {
    type Error = Infallible;

    async fn sub_step_1(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_1: {:?}", self.arg));
        Ok(())
    }

    async fn sub_step_2(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_2: {:?}", self.arg));
        Ok(())
    }
}

impl AsyncSubStory2 for SubStoryImpl<'_> {
    type Error = Infallible;

    async fn sub_step_3(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_3: {:?}", self.arg));
        Ok(())
    }

    async fn sub_step_4(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_4: {:?}", self.arg));
        Ok(())
    }
}

#[allow(dead_code)]
struct MainStoryImpl {
    state: Vec<String>,
}

impl AsyncMainStory for MainStoryImpl {
    type Error = Infallible;

    fn main_step_1(&mut self) -> Result<impl AsyncSubStory<Error = Self::Error>, Self::Error> {
        Ok(SubStoryImpl {
            state: &mut self.state,
            arg: None,
        })
    }

    fn main_step_2(
        &mut self,
        arg: i32,
    ) -> Result<impl AsyncSubStory2<Error = Self::Error>, Self::Error> {
        Ok(SubStoryImpl {
            state: &mut self.state,
            arg: Some(arg),
        })
    }
}

#[test]
fn test_async() {
    use narrative::story::RunStoryAsync as _;
    let mut env = MainStoryImpl { state: vec![] };
    futures::executor::block_on(MainStoryContext.run_story_async(&mut env)).unwrap();
    assert_eq!(
        env.state,
        vec![
            "sub_step_1: None",
            "sub_step_2: None",
            "sub_step_3: Some(2)",
            "sub_step_4: Some(2)",
        ]
    );
}
