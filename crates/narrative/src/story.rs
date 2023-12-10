use crate::step::Step;

pub trait StoryExt {
    type Step: Step;
    type Steps<'a>: Iterator<Item = &'a mut Self::Step>
    where
        Self: 'a;
    fn narrate(&self);
    fn run_all(&mut self);
    fn steps(&mut self) -> Self::Steps<'_>;
}
