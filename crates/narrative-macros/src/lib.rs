mod story;
mod story_syntax;

use proc_macro2::TokenStream;
use quote::quote;
use story_syntax::ItemStory;
use syn::parse_macro_input;

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
