use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(input: &syn::ItemEnum) -> TokenStream {
    let derive = super::derive_attr::generate();
    let _ = &input.attrs;
    let ident = &input.ident;
    let generics = &input.generics;
    let variants = &input.variants;
    quote! {
        #derive
        pub enum #ident #generics { #variants }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_visibility() {
        let input = syn::parse_quote! {
            enum User {
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::serde::Serialize)]
            #[serde(crate = "narrative::serde")]
            pub enum User {
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_enum_keep_variants_attributes() {
        let input = syn::parse_quote! {
            enum User {
                #[keep_this]
                #[keep_this2]
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::serde::Serialize)]
            #[serde(crate = "narrative::serde")]
            pub enum User {
                #[keep_this]
                #[keep_this2]
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_enum_keep_generics() {
        let input = syn::parse_quote! {
            enum User<T, U> {
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::serde::Serialize)]
            #[serde(crate = "narrative::serde")]
            pub enum User<T, U> {
                Admin,
                Developer(String),
                Normal {
                    id: i32,
                    name: String,
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
