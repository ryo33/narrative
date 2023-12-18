use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{item_story::ItemStory, output::step_fn, Asyncness};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let async_trait = match asyncness {
        Asyncness::Sync => quote! {},
        Asyncness::Async => quote!(#[narrative::async_trait]),
    };
    let ident = match asyncness {
        Asyncness::Sync => input.ident.clone(),
        Asyncness::Async => format_ident!("Async{}", input.ident),
    };
    let steps = input.items.iter().filter_map(|item| match item {
        crate::item_story::StoryItem::Step(step) => {
            Some(step_fn::generate(step, asyncness, Some(quote! {Ok(())})))
        }
        _ => None,
    });
    quote! {
        #async_trait
        #[allow(unused_variables)]
        impl #ident for narrative::environment::DummyEnvironment {
            type Error = std::convert::Infallible;
            #(#steps)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            impl UserStory for narrative::environment::DummyEnvironment {
                type Error = std::convert::Infallible;
                fn step1(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
                fn step2(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_async() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Async);
        let expected = quote! {
            #[narrative::async_trait]
            impl AsyncUserStory for narrative::environment::DummyEnvironment {
                type Error = std::convert::Infallible;
                async fn step1(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
                async fn step2(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
