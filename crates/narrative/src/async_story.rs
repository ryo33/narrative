// #[async_trait::async_trait]
// pub trait AsyncStory {
//     type Story;
//     type StepId: StepId;
//     type Step: Step;
//     type Steps: Steps<Step = Self::Step>;
//     type Error: std::error::Error;
//     // This is not &str for future extensibility.
//     /// Returns the title of the story.
//     fn story_title() -> String;
//     /// Returns the identifier of the story.
//     fn story_ident() -> &'static str;
//     /// Returns the steps of the story.
//     fn steps() -> Self::Steps;
//     /// Run all steps in the story. It's a shortcut for iterating over the steps.
//     async fn run_all(story: Self::Story) -> Result<(), RunAllError<Self::StepId, Self::Error>>;
// }
//
// // Fields are public because this type is just a replacement of (StepId, S::Error).
// /// A pair of a step identifier and an error returned by the step.
// pub struct RunAllError<I: StepId, E> {
//     /// The identifier of the step that failed.
//     pub step_id: I,
//     /// The error returned by the step.
//     pub error: E,
// }
