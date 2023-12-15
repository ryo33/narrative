mod narrate;
mod run_all;
mod steps;

use proc_macro2::TokenStream;
use quote::quote;

use crate::story_syntax::ItemStory;

pub fn generate(input: &ItemStory) -> TokenStream {
    quote! {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                fn step1();
                fn step2();
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            impl narrative::story::StoryExt for User {
                type Step = UserStoryStep;
                type Steps<'a> = UserStorySteps<'a>;
                where
                    Self: 'a;
                fn narrate(&self) {
                }
                fn run_all(&mut self);
                fn steps(&mut self) -> Self::Steps<'_>;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
