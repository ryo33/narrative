use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(input: &syn::ItemStruct) -> TokenStream {
    let derive = super::derive_attr::generate();
    let _ = &input.attrs;
    let ident = &input.ident;
    let generics = &input.generics;
    let fields = match &input.fields {
        syn::Fields::Named(named) => {
            let iter = named.named.iter().map(|field| {
                let attrs = &field.attrs;
                let ident = &field.ident;
                let ty = &field.ty;
                quote! {
                    #(#attrs)*
                    pub #ident: #ty,
                }
            });
            quote!({ #(#iter)* })
        }
        syn::Fields::Unnamed(unnamed) => {
            let iter = unnamed.unnamed.iter().enumerate().map(|(_, field)| {
                let attrs = &field.attrs;
                let ty = &field.ty;
                quote! {
                    #(#attrs)*
                    pub #ty
                }
            });
            quote!(( #(#iter),* );)
        }
        syn::Fields::Unit => quote!(;),
    };
    quote! {
        #derive
        pub struct #ident #generics #fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_visibility_unit() {
        let input = syn::parse_quote! {
            struct User;
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User;
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_visibility_named_fields() {
        let input = syn::parse_quote! {
            pub struct User {
                id: i32,
                name: String,
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User {
                pub id: i32,
                pub name: String,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_visibility_unnamed_fields() {
        let input = syn::parse_quote! {
            pub struct User(i32, String);
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User(pub i32, pub String);
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_keep_member_named_attributes() {
        let input = syn::parse_quote! {
            pub struct User {
                #[keep_this]
                #[keep_this2]
                id: i32,
                name: String,
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User {
                #[keep_this]
                #[keep_this2]
                pub id: i32,
                pub name: String,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_keep_member_unnamed_attributes() {
        let input = syn::parse_quote! {
            pub struct User(#[keep_this] #[keep_this2] i32, String);
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User(#[keep_this] #[keep_this2] pub i32, pub String);
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_keep_generics() {
        let input = syn::parse_quote! {
            pub struct User<T, U> {
                id: i32,
                name: String,
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            #[derive(Clone, Debug, PartialEq, narrative::Serialize)]
            pub struct User<T, U> {
                pub id: i32,
                pub name: String,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
