use crate::step::{DynStep, Step};

/// A trait for handing a story in general.
// `&self` is not actually used, and is for future compatibility and friendly API.
pub trait StoryContext: Sized {
    type Step: Step + 'static;
    /// Returns the title of the story.
    fn story_title(&self) -> String;
    /// Returns the identifier of the story.
    fn story_id(&self) -> &'static str;
    fn consts(&self) -> impl Iterator<Item = impl StoryConst + 'static> + 'static;
    /// Returns the steps of the story.
    fn steps(&self) -> impl Iterator<Item = Self::Step> + 'static;
}

pub trait StoryConst: Clone + std::fmt::Debug {
    /// Returns the name of the constant value.
    fn name(&self) -> &'static str;
    /// Returns the type of the constant value.
    fn ty(&self) -> &'static str;
    /// Returns the real expression of the constant value.
    fn expr(&self) -> &'static str;
    /// Returns the debug representation of the value.
    fn debug_value(&self) -> String;
    /// Serializes the value to the given serializer.
    fn serialize_value(&self) -> impl serde::Serialize + 'static;
}

pub type BoxedStoryContext = Box<dyn DynStoryContext>;

mod private {
    pub trait SealedDynStoryContext {}
    pub trait SealedDynStoryConst {}
}

pub trait DynStoryContext: private::SealedDynStoryContext {
    fn story_title(&self) -> String;
    fn story_id(&self) -> &'static str;
    fn consts(&self) -> Box<dyn Iterator<Item = Box<dyn DynStoryConst>>>;
    fn steps(&self) -> Box<dyn Iterator<Item = Box<dyn DynStep>>>;
}

pub trait DynStoryConst: private::SealedDynStoryConst {
    fn name(&self) -> &'static str;
    fn ty(&self) -> &'static str;
    fn expr(&self) -> &'static str;
    fn debug_value(&self) -> String;
    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize>;
}

impl<T: StoryContext> private::SealedDynStoryContext for T {}
impl<T: StoryConst> private::SealedDynStoryConst for T {}

impl<T: StoryContext + private::SealedDynStoryContext> DynStoryContext for T {
    fn story_title(&self) -> String {
        StoryContext::story_title(self)
    }

    fn story_id(&self) -> &'static str {
        StoryContext::story_id(self)
    }

    fn consts(&self) -> Box<dyn Iterator<Item = Box<dyn DynStoryConst>>> {
        Box::new(StoryContext::consts(self).map(|c| Box::new(c) as Box<dyn DynStoryConst>))
    }

    fn steps(&self) -> Box<dyn Iterator<Item = Box<dyn DynStep>>> {
        Box::new(StoryContext::steps(self).map(|s| Box::new(s) as Box<dyn DynStep>))
    }
}

impl<T: StoryConst + private::SealedDynStoryConst> DynStoryConst for T {
    fn name(&self) -> &'static str {
        StoryConst::name(self)
    }

    fn ty(&self) -> &'static str {
        StoryConst::ty(self)
    }

    fn expr(&self) -> &'static str {
        StoryConst::expr(self)
    }

    fn debug_value(&self) -> String {
        StoryConst::debug_value(self)
    }

    fn serialize_value(&self) -> Box<dyn erased_serde::Serialize> {
        Box::new(StoryConst::serialize_value(self))
    }
}
