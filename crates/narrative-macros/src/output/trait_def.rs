use proc_macro2::TokenStream;
use quote::quote;

use crate::story_syntax::ItemStory;

pub fn generate(input: &ItemStory) -> TokenStream {
    let ident = &input.ident;
    let items = input.items.iter().filter_map(|item| match item {
        crate::story_syntax::StoryItem::Step(step) => {
            let step_fn = crate::output::step_fn::generate(step);
            Some(quote! {
                #step_fn
            })
        }
        _ => None,
    });
    quote! {
        pub trait #ident {
            type Error: std::error::Error;
            #(#items)*
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
            trait User {
                fn step1();
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            pub trait User {
                type Error: std::error::Error;
                fn step1(&mut self) -> Result<(), Self::Error>;
                fn step2(&mut self) -> Result<(), Self::Error>;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_trait_invalid_trait_items() {
        let input = syn::parse_quote! {
            trait User {
                struct UserId(String);
                enum User {
                    Admin,
                    Developer(String),
                    Normal {
                        id: UserId,
                        name: String,
                    }
                }
                let user_id = UserId("123".to_string());
                fn step1(user_id: UserId);
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            pub trait User {
                type Error: std::error::Error;
                fn step1(&mut self, user_id: UserId) -> Result<(), Self::Error>;
                fn step2(&mut self) -> Result<(), Self::Error>;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
