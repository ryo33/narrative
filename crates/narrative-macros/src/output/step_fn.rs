use proc_macro2::TokenStream;
use quote::quote;

use crate::{Asyncness, item_story::StoryStep};

/// This does not emits `;` or body.
pub(crate) fn generate(step: &StoryStep, asyncness: Asyncness) -> TokenStream {
    let fn_name = &step.inner.sig.ident;
    let other_attrs = &step.other_attrs;
    let inputs_tokens = step
        .inner
        .sig
        .inputs
        .iter()
        .filter(|input| matches!(input, syn::FnArg::Typed(_)));

    let bounds = if asyncness == Asyncness::Async {
        quote!(+ Send)
    } else {
        quote!()
    };

    // Check if this is a sub-story step
    if let Some(sub_story_path) = step.sub_story_path() {
        let path = sub_story_path.path();
        let async_path = sub_story_path.async_path();
        // Generate different outputs based on asyncness
        let trait_name = match asyncness {
            Asyncness::Sync => quote!(#path),
            Asyncness::Async => quote!(#async_path),
        };

        quote! {
            #(#other_attrs)*
            fn #fn_name(&mut self #(,#inputs_tokens)*) -> Result<impl #trait_name<Error = Self::Error> #bounds, Self::Error>
        }
    } else {
        // Regular step function
        let output = match asyncness {
            Asyncness::Sync => quote!(Result<(), Self::Error>),
            Asyncness::Async => {
                quote!(impl std::future::Future<Output = Result<(), Self::Error>> + Send)
            }
        };

        quote! {
            #(#other_attrs)*
            fn #fn_name(&mut self #(,#inputs_tokens)*) -> #output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate_step_fn_blank() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            fn step1(&mut self) -> Result<(), Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_step_fn_with_inputs() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1(a: i32, b: i32);
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_ignore_receiver() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1(&self, a: i32, b: i32);
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            fn step1(&mut self, a: i32, b: i32) -> Result<(), Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_async() {
        let item_story = syn::parse_quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(&item_story, Asyncness::Async);
        let expected = quote! {
            fn step1(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_substory_step_fn() {
        let item_story = syn::parse_quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub();
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            fn step_with_sub(&mut self) -> Result<impl SubStory<Error = Self::Error>, Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_substory_step_fn_with_inputs() {
        let item_story = syn::parse_quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub(arg: i32);
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            fn step_with_sub(&mut self, arg: i32) -> Result<impl SubStory<Error = Self::Error>, Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_substory_step_fn_async() {
        let item_story = syn::parse_quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub();
        };
        let actual = generate(&item_story, Asyncness::Async);
        let expected = quote! {
            fn step_with_sub(&mut self) -> Result<impl AsyncSubStory<Error = Self::Error> + Send, Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_step_fn_with_other_attrs() {
        let item_story = syn::parse_quote! {
            /// This is a step
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(&item_story, Asyncness::Sync);
        let expected = quote! {
            /// This is a step
            fn step1(&mut self) -> Result<(), Self::Error>
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_step_fn_with_other_attrs_async() {
        let item_story = syn::parse_quote! {
            /// This is a step
            #[step("Step 1")]
            fn step1();
        };
        let actual = generate(&item_story, Asyncness::Async);
        let expected = quote! {
            /// This is a step
            fn step1(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
