use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(input: &syn::ItemTrait) -> TokenStream {
    let ident = &input.ident;
    let generics = &input.generics;
    let items = &input.items;
    quote! {
        pub trait #ident #generics {
            #(#items)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_trait_visibility() {
        let input = syn::parse_quote! {
            trait UserId {
                fn new_v4() -> Self;
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            pub trait UserId {
                fn new_v4() -> Self;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_trait_generics() {
        let input = syn::parse_quote! {
            trait UserId<T> {
                fn new_v4() -> Self;
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            pub trait UserId<T> {
                fn new_v4() -> Self;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
