use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    extract_types_for_assertion::extract_types_for_assertion,
    item_story::{ItemStory, StoryItem},
};

pub fn generate(input: &ItemStory) -> TokenStream {
    let story_name = &input.ident;
    let local_type_trait = format_ident!("{}LocalType", story_name);

    let assertions = input.items.iter().map(|item| match item {
        StoryItem::Step(step) => {
            let args = &step.inner.sig.inputs;
            let types: Vec<_> = args
                .iter()
                .filter_map(|arg| match arg {
                    syn::FnArg::Typed(pat_type) => Some(&*pat_type.ty),
                    _ => None,
                })
                .flat_map(extract_types_for_assertion)
                .collect();

            let assertions = types.iter().map(|ty| {
                quote! {
                    assert_local_type::<#ty>();
                }
            });
            quote! {
                #(#assertions)*
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
}
