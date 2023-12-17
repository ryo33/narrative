// &self in these methods are not necessary but it's for future extensibility and friendly API.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    item_story::{ItemStory, StoryItem},
    Asyncness,
};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = &input.ident;
    let steps = input.items.iter().filter_map(|item| {
        let StoryItem::Step(step) = item else {
            return None;
        };
        let step_name = &step.inner.sig.ident;
        Some(quote! {
            pub fn #step_name(&self) -> Step<steps::#step_name<T, T::Error>> {
                Default::default()
            }
        })
    });
    quote! {
        #[derive(Default)]
        pub struct StoryContext<T: #ident> {
            phantom: std::marker::PhantomData<T>,
        }
        impl <T: #ident> StoryContext<T> {
            #(#steps)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            #[derive(Default)]
            pub struct StoryContext<T: UserStory> {
                phantom: std::marker::PhantomData<T>,
            }
            impl <T: UserStory> StoryContext<T> {
                pub fn step1(&self) -> Step<steps::step1<T, T::Error>> {
                    Default::default()
                }
                pub fn step2(&self) -> Step<steps::step2<T, T::Error>> {
                    Default::default()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
