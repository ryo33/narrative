use proc_macro2::TokenStream;
use quote::quote;

use crate::item_story::{ItemStory, StoryItem};

// FIXME: Rewrite completely without complex logic and in the same approach as
// local_type_assertions.
pub fn generate(input: &ItemStory) -> TokenStream {
    // (all step args) - (step attr args)
    let usages = input
        .items
        .iter()
        .filter_map(|item| match item {
            StoryItem::Step(step) => Some(step),
            _ => None,
        })
        .flat_map(|step| {
            step.inner
                .sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    syn::FnArg::Typed(pat_type) => Some(&pat_type.pat),
                    syn::FnArg::Receiver(_) => None,
                })
                .filter(|pat| {
                    if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                        // If there is no attr arg with the same name, it uses an assignment.
                        step.attr
                            .args
                            .iter()
                            .all(|arg| pat_ident.ident != arg.ident)
                    } else {
                        // If fn arg is a pattern, it always uses an assignment.
                        true
                    }
                })
        })
        .collect::<Vec<_>>();
    let unused_assignments = input
        .items
        .iter()
        .filter_map(|item| match item {
            crate::item_story::StoryItem::Let(assignment) => Some(assignment),
            _ => None,
        })
        .filter(|assignment| {
            usages
                .iter()
                .all(|usage| *assignment.pat.as_ref() != *usage.as_ref())
        });
    quote! {
        fn _unused_assignments() {
            #(#unused_assignments;)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            fn _unused_assignments() {
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_used() {
        let story_syntax = syn::parse_quote! {
            trait User {
                let id = UserId::new();
                let name = "Alice";
                #[step("Step 1")]
                fn step1();
                #[step("Step 1")]
                fn step2(id: UserId, name: &str);
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            fn _unused_assignments() {
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_not_used() {
        let story_syntax = syn::parse_quote! {
            trait User {
                let id = UserId::new();
                let name = "Alice";
                #[step("Step 1")]
                fn step1();
                #[step("Step 1", name = "Bob")]
                fn step2(id: UserId, name: &str);
            }
        };
        let actual = generate(&story_syntax);
        let expected = quote! {
            fn _unused_assignments() {
                let name = "Alice";
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_used_pattern() {}

    #[test]
    fn test_not_used_pattern() {}
}
