use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input, ItemTrait};

#[proc_macro_attribute]
pub fn story(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as syn::ItemTrait);
    story_impl(item).into()
}

fn story_impl(item: ItemTrait) -> TokenStream {
    quote! {}
}
