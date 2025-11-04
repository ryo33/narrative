use quote::format_ident;

use crate::extract_types_for_assertion::extract_types_for_assertion;

/// Extract all field types from a struct or enum
fn extract_field_types(input_item: &syn::Item) -> Vec<&syn::Type> {
    match input_item {
        syn::Item::Struct(item_struct) => {
            let field_types: Vec<_> = match &item_struct.fields {
                syn::Fields::Named(fields) => fields.named.iter().map(|f| &f.ty).collect(),
                syn::Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| &f.ty).collect(),
                syn::Fields::Unit => vec![],
            };
            field_types
                .into_iter()
                .flat_map(extract_types_for_assertion)
                .collect()
        }
        syn::Item::Enum(item_enum) => {
            let field_types: Vec<_> = item_enum
                .variants
                .iter()
                .flat_map(|variant| match &variant.fields {
                    syn::Fields::Named(fields) => {
                        fields.named.iter().map(|f| &f.ty).collect::<Vec<_>>()
                    }
                    syn::Fields::Unnamed(fields) => {
                        fields.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>()
                    }
                    syn::Fields::Unit => vec![],
                })
                .collect();
            field_types
                .into_iter()
                .flat_map(extract_types_for_assertion)
                .collect()
        }
        _ => vec![],
    }
}

pub(crate) fn generate(
    story_name: &syn::Ident,
    input_item: &syn::Item,
) -> proc_macro2::TokenStream {
    let (type_name, generics) = match &input_item {
        syn::Item::Struct(item) => (&item.ident, &item.generics),
        syn::Item::Enum(item) => (&item.ident, &item.generics),
        _ => {
            return syn::Error::new_spanned(
                input_item,
                "local_type_for can only be applied to structs or enums",
            )
            .to_compile_error();
        }
    };

    let local_type_trait = format_ident!("{}LocalType", story_name);

    // Add StoryOwnedType bound to all type parameters
    let mut impl_generics = generics.clone();
    impl_generics.type_params_mut().for_each(|param| {
        param
            .bounds
            .push(syn::parse_quote!(narrative::StoryOwnedType));
        param.bounds.push(syn::parse_quote!(#local_type_trait));
    });

    // Type parameters without bounds for usage
    let mut type_generics = generics.clone();
    type_generics.type_params_mut().for_each(|param| {
        param.bounds.clear();
    });

    // Extract field types and generate assertions
    let field_types = extract_field_types(input_item);
    let assertions = field_types
        .iter()
        .map(|ty| quote::quote!(assert_local_type::<#ty>();));

    // Generate unique assertion function name based on type name
    let assertion_fn_name = format_ident!("_local_type_assertions_{}", type_name);

    quote::quote! {
        #input_item

        // Implement StoryOwnedType for this type
        // This will conflict if #[local_type_for] is applied to the same type twice,
        // preventing a data type from being a local type for multiple stories
        impl #impl_generics narrative::StoryOwnedType for #type_name #type_generics {}

        // Implement StoryLocalType for this type
        impl #impl_generics #local_type_trait for #type_name #type_generics {}

        // Type assertions for field types
        #[allow(non_snake_case)]
        fn #assertion_fn_name() {
            fn assert_local_type<T: #local_type_trait>() {}
            #(#assertions)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn test_struct_with_fields() {
        let story_name: syn::Ident = syn::parse_quote!(User);
        let input: syn::Item = syn::parse_quote! {
            struct UserId {
                id: u64,
                tags: Vec<String>,
            }
        };
        let actual = generate(&story_name, &input);
        let expected = quote! {
            struct UserId {
                id: u64,
                tags: Vec<String>,
            }

            impl narrative::StoryOwnedType for UserId {}

            impl UserLocalType for UserId {}

            #[allow(non_snake_case)]
            fn _local_type_assertions_UserId() {
                fn assert_local_type<T: UserLocalType>() {}
                assert_local_type::<u64>();
                assert_local_type::<String>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_enum_with_variants() {
        let story_name: syn::Ident = syn::parse_quote!(User);
        let input: syn::Item = syn::parse_quote! {
            enum UserEvent {
                Created(UserId),
                Updated { id: UserId, name: String },
            }
        };
        let actual = generate(&story_name, &input);
        let expected = quote! {
            enum UserEvent {
                Created(UserId),
                Updated { id: UserId, name: String },
            }

            impl narrative::StoryOwnedType for UserEvent {}

            impl UserLocalType for UserEvent {}

            #[allow(non_snake_case)]
            fn _local_type_assertions_UserEvent() {
                fn assert_local_type<T: UserLocalType>() {}
                assert_local_type::<UserId>();
                assert_local_type::<UserId>();
                assert_local_type::<String>();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
