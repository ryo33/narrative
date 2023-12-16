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

/// This is not a real attribute macro, and it's just for provide meta data to further macros.
#[proc_macro_attribute]
pub fn step(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    input
}

#[proc_macro_attribute]
pub fn story(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as StoryAttr);
    let story = parse_macro_input!(input as ItemStory);
    process_story(attr, story).into()
}

fn process_story(attr: StoryAttr, story: ItemStory) -> TokenStream {
    output::generate(&attr, &story)
}

pub(crate) enum Asyncness {
    Sync,
    Async,
}
