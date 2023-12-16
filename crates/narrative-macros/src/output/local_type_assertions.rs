use proc_macro2::TokenStream;
use quote::quote;

use crate::item_story::{ItemStory, StoryItem};

pub fn generate(input: &ItemStory) -> TokenStream {
    let assertions = input.items.iter().map(|item| match item {
        StoryItem::Step(step) => {
            let args = &step.inner.sig.inputs;
            let args = args.iter().map(|arg| {
                let ty = match arg {
                    syn::FnArg::Typed(pat_type) => &pat_type.ty,
                    _ => return Default::default(),
                };
                if let syn::Type::Path(path) = ty.as_ref() {
                    if path
                        .path
                        .segments
                        .first()
                        .expect("not empty type path")
                        .ident
                        == "Self"
                    {
                        return Default::default();
                    }
                }
                quote! {
                    assert_local_type::<#ty>();
                }
            });
            quote! {
                #(#args)*
            }
        }
        StoryItem::Struct(item) => {
            let fields = &item.fields;
            let fields = fields.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    assert_local_type::<#ty>();
                }
            });
            quote! {
                #(#fields)*
            }
        }
        StoryItem::Enum(item) => {
            let variants = &item.variants;
            let variants = variants.iter().map(|variant| {
                let fields = &variant.fields;
                let fields = fields.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        assert_local_type::<#ty>();
                    }
                });
                quote! {
                    #(#fields)*
                }
            });
            quote! {
                #(#variants)*
            }
        }
        _ => Default::default(),
    });
    quote! {
        fn _local_type_assertions() {
            fn assert_local_type<T: LocalType>() {}
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
                fn assert_local_type<T: LocalType>() {}
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
                fn assert_local_type<T: LocalType>() {}
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
                fn assert_local_type<T: LocalType>() {}
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_named_fields() {
        let input = syn::parse_quote! {
            trait User {
                struct User {
                    id: UserId,
                    name: &str,
                }
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: LocalType>() {}
                assert_local_type::<UserId>();
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_unnamed_fields() {
        let input = syn::parse_quote! {
            trait User {
                struct User(UserId, &str);
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: LocalType>() {}
                assert_local_type::<UserId>();
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_enum() {
        let input = syn::parse_quote! {
            trait User {
                enum User {
                    Admin {
                        name: &str,
                    },
                    Developer {
                        id: DeveloperId,
                        name: &str,
                    },
                    Normal(UserId, &str),
                }
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&input);
        let expected = quote! {
            fn _local_type_assertions() {
                fn assert_local_type<T: LocalType>() {}
                assert_local_type::<&str>();
                assert_local_type::<DeveloperId>();
                assert_local_type::<&str>();
                assert_local_type::<UserId>();
                assert_local_type::<&str>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
