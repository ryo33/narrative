// The generated trait is used as a super trait of the story trait, and provides `Self::*` items
// for coupled data types.
// Sealing the trait is not necessary because we have blanket impls of this for all types.

use proc_macro2::TokenStream;
use quote::quote;

use crate::item_story::{ItemStory, StoryItem};

pub fn generate(input: &ItemStory) -> TokenStream {
    let types = input.items.iter().filter_map(|item| {
        let (ident, generics) = match item {
            StoryItem::Struct(item) => (&item.ident, &item.generics),
            StoryItem::Enum(item) => (&item.ident, &item.generics),
            _ => return None,
        };
        Some(quote! {
            type #ident #generics;
        })
    });
    let types_assigns = input.items.iter().filter_map(|item| {
        let (ident, generics) = match item {
            StoryItem::Struct(item) => (&item.ident, &item.generics),
            StoryItem::Enum(item) => (&item.ident, &item.generics),
            _ => return None,
        };
        let mut no_bound = generics.clone();
        no_bound.type_params_mut().for_each(|param| {
            param.bounds.clear();
        });
        Some(quote! {
            type #ident #generics = #ident #no_bound;
        })
    });
    quote! {
        pub trait BaseTrait {
            #(#types)*
            type Context: narrative::story::StoryContext;
        }
        impl<T> BaseTrait for T {
            #(#types_assigns)*
            type Context = StoryContext;
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
            trait User {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            pub trait BaseTrait {
                type Context: narrative::story::StoryContext;
            }
            impl<T> BaseTrait for T {
                type Context = StoryContext;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_with_data_types() {
        let story_syntax = syn::parse_quote! {
            trait User {
                struct UserId;
                enum UserKind {}
                trait UserTrait {}
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            pub trait BaseTrait {
                type UserId;
                type UserKind;
                type Context: narrative::story::StoryContext;
            }
            impl<T> BaseTrait for T {
                type UserId = UserId;
                type UserKind = UserKind;
                type Context = StoryContext;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_with_generics() {
        let story_syntax = syn::parse_quote! {
            trait User {
                struct UserId<T>(T);
                enum UserKind<T: Clone> {}
                trait UserTrait<T> {}
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            pub trait BaseTrait {
                type UserId<T>;
                type UserKind<T: Clone>;
                type Context: narrative::story::StoryContext;
            }
            impl<T> BaseTrait for T {
                type UserId<T> = UserId<T>;
                type UserKind<T: Clone> = UserKind<T>;
                type Context = StoryContext;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
