// The generated trait is used as a super trait of the story trait, and provides `Self::*` items
// for coupled data types.
// Sealing the trait is not necessary because we have blanket impls of this for all types.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    item_story::{story_const::StoryConst, ItemStory},
    Asyncness,
};

pub fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let trait_ident = match asyncness {
        Asyncness::Sync => quote!(BaseTrait),
        Asyncness::Async => quote!(AsyncBaseTrait),
    };
    let story_trait_ident = match asyncness {
        Asyncness::Sync => input.ident.clone(),
        Asyncness::Async => format_ident!("Async{}", input.ident),
    };
    let consts = input.consts().map(|StoryConst { raw, .. }| {
        let ident = &raw.ident;
        let ty = &raw.ty;
        Some(quote! {
            const #ident: #ty;
        })
    });
    let consts_assigns = input.consts().map(|StoryConst { raw, .. }| raw);
    quote! {
        pub trait #trait_ident {
            #(#consts)*
            type Context: narrative::story::StoryContext;
            const CONTEXT: Self::Context;
        }
        impl<T: #story_trait_ident> #trait_ident for T {
            #(#consts_assigns)*
            type Context = StoryContext;
            const CONTEXT: StoryContext = StoryContext;
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
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            pub trait BaseTrait {
                type Context: narrative::story::StoryContext;
                const CONTEXT: Self::Context;
            }
            impl<T: User> BaseTrait for T {
                type Context = StoryContext;
                const CONTEXT: StoryContext = StoryContext;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }


    #[test]
    fn test_async() {
        let story_syntax = syn::parse_quote! {
            trait User {
                #[step("Step 1")]
                async fn step1();
                #[step("Step 2")]
                async fn step2();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Async);
        let expected = quote! {
            pub trait AsyncBaseTrait {
                type Context: narrative::story::StoryContext;
                const CONTEXT: Self::Context;
            }
            impl<T: AsyncUser> AsyncBaseTrait for T {
                type Context = StoryContext;
                const CONTEXT: StoryContext = StoryContext;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
