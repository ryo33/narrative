use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::item_story::ItemStory;

pub fn generate(input: &ItemStory) -> TokenStream {
    let story_name = &input.ident;
    let local_type_trait = format_ident!("{}LocalType", story_name);

    quote! {
        #[diagnostic::on_unimplemented(
            message = "the type `{Self}` cannot be used in this story",
            label = "this type is not allowed in stories",
            note = "only types from the standard library or types defined with #[local_type_for] are allowed"
        )]
        pub trait #local_type_trait {}

        // Blanket impl for standard library types that already implement IndependentType
        #[diagnostic::do_not_recommend]
        impl<T: narrative::IndependentType> #local_type_trait for T {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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
                note = "only types from the standard library or types defined with #[local_type_for] are allowed"
            )]
            pub trait UserLocalType {}

            #[diagnostic::do_not_recommend]
            impl<T: narrative::IndependentType> UserLocalType for T {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
