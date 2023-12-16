use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, output::step_fn, Asyncness};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = &input.ident;
    let steps = input.items.iter().filter_map(|item| match item {
        crate::item_story::StoryItem::Step(step) => Some(step_fn::generate(step, None)),
        _ => None,
    });
    quote! {
        pub trait #ident: BaseTrait {
            type Error: std::error::Error;
            #(#steps)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_trait_visibility() {
        let input = syn::parse_quote! {
            trait UserStory {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2", user_id = UserId::new())]
                fn step2(user_id: UserId);
            }
        };
        let actual = generate(&input, Asyncness::Sync);
        let expected = quote! {
            pub trait UserStory: BaseTrait {
                type Error: std::error::Error;
                fn step1(&mut self) -> Result<(), Self::Error>;
                fn step2(&mut self, user_id: UserId) -> Result<(), Self::Error>;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
