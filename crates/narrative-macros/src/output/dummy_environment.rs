use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, output::step_fn};

pub fn generate(input: &ItemStory) -> TokenStream {
    let ident = &input.ident;
    let steps = input.items.iter().filter_map(|item| match item {
        crate::item_story::StoryItem::Step(step) => {
            Some(step_fn::generate(step, Some(quote! {Ok(())})))
        }
        _ => None,
    });
    quote! {
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
        let actual = generate(&story_syntax);
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
}
