mod derive_attr;
mod enum_def;
mod step_fn;
mod step_iter;
mod story_ext;
mod struct_def;
mod trait_def;

use proc_macro2::TokenStream;
use quote::quote;

use crate::story_syntax::ItemStory;

pub fn generate(story_syntax: &ItemStory) -> TokenStream {
    let trait_def = trait_def::generate(story_syntax);
    quote! {
        #trait_def
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate() {
        let story_syntax = syn::parse_quote! {
            trait User {
                fn step1();
                fn step2();
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            pub trait User {
                type Error: std::error::Error;
                fn step1(&mut self) -> Result<(), Self::Error>;
                fn step2(&mut self) -> Result<(), Self::Error>;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
