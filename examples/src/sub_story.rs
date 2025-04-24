use std::convert::Infallible;

#[narrative::story("This is a sub story")]
trait SubStory {
    #[step("sub_step_1")]
    fn sub_step_1();
    #[step("sub_step_2")]
    fn sub_step_2();
}

#[narrative::story("This is a main story")]
trait MainStory {
    #[step(story: SubStory, "do sub story")]
    fn main_step_1();
    #[step(story: SubStory, "do sub story with args", arg = 2)]
    fn main_step_2(arg: i32);
}

#[allow(dead_code)]
struct SubStoryImpl<'a> {
    state: &'a mut Vec<String>,
    arg: Option<i32>,
}

impl SubStory for SubStoryImpl<'_> {
    type Error = Infallible;

    fn sub_step_1(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_1: {:?}", self.arg));
        Ok(())
    }

    fn sub_step_2(&mut self) -> Result<(), Self::Error> {
        self.state.push(format!("sub_step_2: {:?}", self.arg));
        Ok(())
    }
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

#[allow(dead_code)]
struct MainStoryImpl {
    state: Vec<String>,
}

impl MainStory for MainStoryImpl {
    type Error = Infallible;

    fn main_step_1(&mut self) -> Result<impl SubStory<Error = Self::Error>, Self::Error> {
        Ok(SubStoryImpl {
            state: &mut self.state,
            arg: None,
        })
    }

    fn main_step_2(&mut self, arg: i32) -> Result<impl SubStory<Error = Self::Error>, Self::Error> {
        Ok(SubStoryImpl {
            state: &mut self.state,
            arg: Some(arg),
        })
    }
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
    ) -> Result<impl AsyncSubStory<Error = Self::Error>, Self::Error> {
        Ok(SubStoryImpl {
            state: &mut self.state,
            arg: Some(arg),
        })
    }
}

#[test]
fn test() {
    let mut story = MainStoryImpl { state: vec![] };
    story.run_all().unwrap();
    assert_eq!(
        story.state,
        vec![
            "sub_step_1: None",
            "sub_step_2: None",
            "sub_step_1: Some(2)",
            "sub_step_2: Some(2)",
        ]
    );
}

#[test]
fn test_async() {
    let mut story = MainStoryImpl { state: vec![] };
    futures::executor::block_on(story.run_all_async()).unwrap();
    assert_eq!(
        story.state,
        vec![
            "sub_step_1: None",
            "sub_step_2: None",
            "sub_step_1: Some(2)",
            "sub_step_2: Some(2)",
        ]
    );
}
