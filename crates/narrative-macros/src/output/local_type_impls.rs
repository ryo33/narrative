use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use crate::item_story::{ItemStory, StoryItem};

pub fn generate(input: &ItemStory) -> TokenStream {
    let local_type_impls = input.items.iter().map(|item| {
        let (ty, generics) = match item {
            StoryItem::Struct(item) => (&item.ident, &item.generics),
            StoryItem::Enum(item) => (&item.ident, &item.generics),
            _ => return Default::default(),
        };
        let mut bounds = generics.clone();
        bounds.type_params_mut().for_each(|param| {
            param
                .bounds
                .push(syn::TypeParamBound::Trait(parse_quote!(LocalType)));
        });
        let mut names = generics.clone();
        names.type_params_mut().for_each(|param| {
            param.bounds.clear();
        });
        quote! {
            impl #bounds LocalType for #ty #names {}
        }
    });
    quote! {
        #[diagnostic::on_unimplemented(
            message = "the type `{Self}` cannot be used in this story",
            label = "this type is not allowed in stories",
            note = "only types from the standard library or types defined within the story are allowed"
        )]
        trait LocalType {}
        #[diagnostic::do_not_recommend]
        impl <T: narrative::IndependentType> LocalType for T {}
        #(#local_type_impls)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_independent_type() {
        let input = syn::parse_quote! {
            trait User {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[diagnostic::on_unimplemented(
                message = "the type `{Self}` cannot be used in this story",
                label = "this type is not allowed in stories",
                note = "only types from the standard library or types defined within the story are allowed"
            )]
            trait LocalType {}
            #[diagnostic::do_not_recommend]
            impl <T: narrative::IndependentType> LocalType for T {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_impl_for_data_types() {
        let input = syn::parse_quote! {
            trait User {
                pub struct UserId;
                pub enum UserKind {}
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[diagnostic::on_unimplemented(
                message = "the type `{Self}` cannot be used in this story",
                label = "this type is not allowed in stories",
                note = "only types from the standard library or types defined within the story are allowed"
            )]
            trait LocalType {}
            #[diagnostic::do_not_recommend]
            impl <T: narrative::IndependentType> LocalType for T {}
            impl LocalType for UserId {}
            impl LocalType for UserKind {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    // Gererics must be bounded by LocalType.
    fn test_impl_for_data_types_with_generics() {
        let input = syn::parse_quote! {
            trait User {
                pub struct UserId<T>(T);
                pub enum UserKind<T: Clone> {}
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[diagnostic::on_unimplemented(
                message = "the type `{Self}` cannot be used in this story",
                label = "this type is not allowed in stories",
                note = "only types from the standard library or types defined within the story are allowed"
            )]
            trait LocalType {}
            #[diagnostic::do_not_recommend]
            impl <T: narrative::IndependentType> LocalType for T {}
            impl <T: LocalType> LocalType for UserId<T> {}
            impl <T: Clone + LocalType> LocalType for UserKind<T> {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
