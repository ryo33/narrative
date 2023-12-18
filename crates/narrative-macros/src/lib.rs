mod error;
mod item_story;
mod no_foreign_type_validation;
mod output;
mod step_attr_syntax;
mod step_usage;
mod story_attr_syntax;

use item_story::ItemStory;
use proc_macro2::TokenStream;
use story_attr_syntax::StoryAttr;
use syn::parse_macro_input;

#[proc_macro_attribute]
/// TODO: Add documentation.
pub fn story(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as StoryAttr);
    let story = parse_macro_input!(input as ItemStory);
    process_story(attr, story).into()
}

// In general, we don't do caching some intermediate results to keep the implementation simple.
// However, we should avoid to have heavy computation in this crate, to keep the story compilation
// fast. So, modules have their own functionality which is simple.
fn process_story(attr: StoryAttr, story: ItemStory) -> TokenStream {
    output::generate(&attr, &story)
}

#[derive(Clone, Copy)]
pub(crate) enum Asyncness {
    Sync,
    Async,
}
