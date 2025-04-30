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

impl quote::ToTokens for Asyncness {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Asyncness::Sync => quote::quote!().to_tokens(tokens),
            Asyncness::Async => quote::quote!(async).to_tokens(tokens),
        }
    }
}

pub(crate) fn collect_format_args(lit_str: &syn::LitStr) -> Vec<String> {
    lit_str
        .value()
        // remove escaped braces
        .split("{{")
        .flat_map(|part| part.split("}}"))
        // iter parts that start with '{' by skipping the first split
        .flat_map(|part| part.split('{').skip(1))
        // take the part before the first '}'
        .filter_map(|part| part.split_once('}').map(|(head, _)| head))
        // remove parts after the first ':'
        .map(|format| {
            format
                .split_once(':')
                .map(|(head, _)| head)
                .unwrap_or(format)
        })
        .map(ToOwned::to_owned)
        .collect()
}
