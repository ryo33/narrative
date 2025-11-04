use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::item_story::{ItemStory, StoryItem};

pub fn generate(input: &ItemStory) -> TokenStream {
    let story_name = &input.ident;
    let local_type_trait = format_ident!("{}LocalType", story_name);

    let assertions = input.items.iter().map(|item| match item {
        StoryItem::Step(step) => {
            let args = &step.inner.sig.inputs;
            let args = args.iter().map(|arg| {
                let ty = match arg {
                    syn::FnArg::Typed(pat_type) => &pat_type.ty,
                    _ => return Default::default(),
                };
                if let syn::Type::Path(path) = ty.as_ref()
                    && path
                        .path
                        .segments
                        .first()
                        .expect("not empty type path")
                        .ident
                        == "Self"
                {
                    return Default::default();
                }
                quote! {
                    assert_local_type::<#ty>();
                }
            });
            quote! {
                #(#args)*
            }
        }
        _ => Default::default(),
    });
    quote! {
        fn _local_type_assertions() {
            fn assert_local_type<T: #local_type_trait>() {}
            #(#assertions)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_full() {
        let input = syn::parse_quote! {
            trait User {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2(id: UserId, name: &str);
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: UserLocalType>() {}
                assert_local_type::<UserId>();
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_in_step_args() {
        let input = syn::parse_quote! {
            trait User {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2(id: UserId, name: &str);
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: UserLocalType>() {}
                assert_local_type::<UserId>();
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_not_include_self_assoc_types() {
        let input = syn::parse_quote! {
            trait User {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2(id: Self::UserId, name: &str);
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: UserLocalType>() {}
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
