mod story;
mod story_syntax;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use story_syntax::ItemStory;
use syn::parse_macro_input;

/// This is not a real attribute macro, and it's just for changing the error message to be more
/// friendly one.
#[proc_macro_attribute]
pub fn step(
    attr: proc_macro::TokenStream,
    _input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let span = attr.into_iter().next().expect("step attr").span().into();
    quote_spanned! {span => compile_error!("step attribute must be used inside a story that is annotated with #[narrative::story]")}
        .into()
}

#[proc_macro_attribute]
pub fn story(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemStory);
    process_story(item).into()
}

fn process_story(story: ItemStory) -> TokenStream {
    quote! {}
}
