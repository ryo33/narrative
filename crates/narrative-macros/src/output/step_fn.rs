use proc_macro2::TokenStream;
use quote::quote;

use crate::item_story::StoryStep;

pub fn generate(item_story: &StoryStep, body: Option<TokenStream>) -> TokenStream {
    let fn_name = &item_story.inner.sig.ident;
    let inputs_tokens = item_story
        .inner
        .sig
        .inputs
        .iter()
        .filter(|input| matches!(input, syn::FnArg::Typed(_)));
    let body = body
        .map(|body| {
            quote! {
                {
                    #body
                }
            }
        })
        .unwrap_or_else(|| quote!(;));
    quote! {
        fn #fn_name(&mut self #(,#inputs_tokens)*) -> Result<(), Self::Error> #body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_step_fn_blank() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(&item_story, None);
        let expected = quote! {
            fn step1(&mut self) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_step_fn_with_inputs() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1(a: i32, b: i32);
        };
        let actual = generate(&item_story, None);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_ignore_receiver() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1(&self, a: i32, b: i32);
        };
        let actual = generate(&item_story, None);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_with_body() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(
            &item_story,
            Some(quote! {
                    println!("Hello, world!");
            }),
        );
        let expected = quote! {
            fn step1(&mut self) -> Result<(), Self::Error> {
                println!("Hello, world!");
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
