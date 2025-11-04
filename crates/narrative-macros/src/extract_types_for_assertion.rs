/// Extracts types that should be asserted from a given type.
///
/// This function recursively extracts types from container types like Vec, Option, Result,
/// arrays, slices, and tuples. For other types, it returns the type itself.
///
/// Types starting with `Self` are filtered out and return an empty vector.
pub fn extract_types_for_assertion(ty: &syn::Type) -> Vec<&syn::Type> {
    // Check if this is a Self type
    if let syn::Type::Path(path) = ty
        && let Some(first_segment) = path.path.segments.first()
        && first_segment.ident == "Self"
    {
        return vec![];
    }

    match ty {
        // Handle references &T
        syn::Type::Reference(type_ref) => {
            match type_ref.elem.as_ref() {
                // Special case: &str should be returned as-is
                syn::Type::Path(path)
                    if path.path.segments.len() == 1 && path.path.segments[0].ident == "str" =>
                {
                    vec![ty]
                }
                // Handle slice &[T] - extract T
                syn::Type::Slice(slice) => extract_types_for_assertion(&slice.elem),
                // For other references, recurse into T
                _ => extract_types_for_assertion(&type_ref.elem),
            }
        }
        // Handle arrays [T; N]
        syn::Type::Array(array) => extract_types_for_assertion(&array.elem),
        // Handle slices [T]
        syn::Type::Slice(slice) => extract_types_for_assertion(&slice.elem),
        // Handle tuples (T1, T2, ...)
        syn::Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .flat_map(extract_types_for_assertion)
            .collect(),
        // Handle path types (Vec<T>, Option<T>, Result<T, E>, etc.)
        syn::Type::Path(type_path) => {
            if let Some(last_segment) = type_path.path.segments.last() {
                let ident = &last_segment.ident;

                // Check for standard library container types
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    let ident_str = ident.to_string();
                    let is_container = matches!(
                        ident_str.as_str(),
                        "Vec"
                            | "Option"
                            | "Result"
                            | "Box"
                            | "Rc"
                            | "Arc"
                            | "Cow"
                            | "HashMap"
                            | "BTreeMap"
                            | "HashSet"
                            | "BTreeSet"
                            | "VecDeque"
                            | "BinaryHeap"
                            | "LinkedList"
                            | "PhantomData"
                            | "LazyLock"
                            | "LazyCell"
                    );

                    if is_container {
                        // Extract generic type arguments
                        return args
                            .args
                            .iter()
                            .filter_map(|arg| {
                                if let syn::GenericArgument::Type(ty) = arg {
                                    Some(ty)
                                } else {
                                    None
                                }
                            })
                            .flat_map(extract_types_for_assertion)
                            .collect();
                    }
                }
            }

            // For non-container types, return the type itself
            vec![ty]
        }
        // For other types, return the type itself
        _ => vec![ty],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn test_extract_vec() {
        let ty: syn::Type = syn::parse_quote!(Vec<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_option() {
        let ty: syn::Type = syn::parse_quote!(Option<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_result() {
        let ty: syn::Type = syn::parse_quote!(Result<UserId, ErrorType>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 2);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(UserId ErrorType).to_string()
        );
    }

    #[test]
    fn test_extract_array() {
        let ty: syn::Type = syn::parse_quote!([UserId; 10]);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_slice_reference() {
        let ty: syn::Type = syn::parse_quote!(&[UserId]);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_tuple() {
        let ty: syn::Type = syn::parse_quote!((UserId, UserName));
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 2);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(UserId UserName).to_string()
        );
    }

    #[test]
    fn test_extract_nested() {
        let ty: syn::Type = syn::parse_quote!(Vec<Option<UserId>>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_complex_nested() {
        let ty: syn::Type = syn::parse_quote!(Result<Vec<(UserId, UserName)>, ErrorType>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 3);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(UserId UserName ErrorType).to_string()
        );
    }

    #[test]
    fn test_extract_str_reference() {
        let ty: syn::Type = syn::parse_quote!(&str);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(&str).to_string());
    }

    #[test]
    fn test_extract_reference_to_option() {
        let ty: syn::Type = syn::parse_quote!(&Option<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_reference_to_vec() {
        let ty: syn::Type = syn::parse_quote!(&Vec<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_reference_to_custom_type() {
        let ty: syn::Type = syn::parse_quote!(&UserId);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_self_type() {
        let ty: syn::Type = syn::parse_quote!(Self);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_self_assoc_type() {
        let ty: syn::Type = syn::parse_quote!(Self::UserId);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_self_in_vec() {
        let ty: syn::Type = syn::parse_quote!(Vec<Self>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_self_in_option() {
        let ty: syn::Type = syn::parse_quote!(Option<Self>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_extract_self_in_result() {
        let ty: syn::Type = syn::parse_quote!(Result<Self, ErrorType>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(ErrorType).to_string()
        );
    }

    #[test]
    fn test_extract_self_in_tuple() {
        let ty: syn::Type = syn::parse_quote!((Self, UserId));
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_box() {
        let ty: syn::Type = syn::parse_quote!(Box<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_rc() {
        let ty: syn::Type = syn::parse_quote!(Rc<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_arc() {
        let ty: syn::Type = syn::parse_quote!(Arc<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_cow() {
        let ty: syn::Type = syn::parse_quote!(Cow<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_hashmap() {
        let ty: syn::Type = syn::parse_quote!(HashMap<String, UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 2);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(String UserId).to_string()
        );
    }

    #[test]
    fn test_extract_btreemap() {
        let ty: syn::Type = syn::parse_quote!(BTreeMap<String, UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 2);
        assert_eq!(
            quote!(#(#result)*).to_string(),
            quote!(String UserId).to_string()
        );
    }

    #[test]
    fn test_extract_hashset() {
        let ty: syn::Type = syn::parse_quote!(HashSet<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_btreeset() {
        let ty: syn::Type = syn::parse_quote!(BTreeSet<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_vecdeque() {
        let ty: syn::Type = syn::parse_quote!(VecDeque<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_nested_containers() {
        let ty: syn::Type = syn::parse_quote!(Arc<Vec<Option<UserId>>>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_binaryheap() {
        let ty: syn::Type = syn::parse_quote!(BinaryHeap<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }

    #[test]
    fn test_extract_linkedlist() {
        let ty: syn::Type = syn::parse_quote!(LinkedList<UserId>);
        let result = extract_types_for_assertion(&ty);
        assert_eq!(result.len(), 1);
        assert_eq!(quote!(#(#result)*).to_string(), quote!(UserId).to_string());
    }
}
