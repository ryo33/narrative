use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(item_story: &syn::TraitItemFn) -> TokenStream {
    let fn_name = &item_story.sig.ident;
    let inputs_tokens = item_story
        .sig
        .inputs
        .iter()
        .filter(|input| matches!(input, syn::FnArg::Typed(_)));
    quote! {
        fn #fn_name(&mut self #(,#inputs_tokens)*) -> Result<(), Self::Error>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_step_fn_blank() {
        let item_story = syn::parse_quote! {
            fn step1();
        };
        let actual = generate(&item_story);
        let expected = quote! {
            fn step1(&mut self) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_step_fn_with_inputs() {
        let item_story = syn::parse_quote! {
            fn step1(a: i32, b: i32);
        };
        let actual = generate(&item_story);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_ignore_receiver() {
        let item_story = syn::parse_quote! {
            fn step1(&self, a: i32, b: i32);
        };
        let actual = generate(&item_story);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
