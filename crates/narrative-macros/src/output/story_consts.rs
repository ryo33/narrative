// important! We don't make story const types camel case, to support rich rename experience on rust-analyzer.
// To avoid name conflict, we use dedicated module for consts.
// enum dispatched by const name

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    item_story::ItemStory, make_static, output::MatchArms, pretty_print_expr, pretty_print_type,
};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let const_names = story
        .consts()
        .map(|item| &item.raw.ident)
        .collect::<Vec<_>>();
    if const_names.is_empty() {
        return quote! {
            /// This is placeholder and never be constructed.
            pub type StoryConst = narrative::story::DynStoryConst;
        };
    }
    let const_variants = story.consts().map(|item| {
        let ident = &item.raw.ident;
        let ty = &item.raw.ty;
        let static_ty = make_static(ty);
        quote!(#ident(#static_ty))
    });

    let const_value_debug_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote!(Self::#ident(value) => value.fmt(f),)
        })
        .collect::<MatchArms>();

    let const_mod_defs = story.consts().map(|item| {
        let ident = &item.raw.ident;
        let ty = pretty_print_type(&item.raw.ty);
        let expr = &item.default.1;
        let expr_str = pretty_print_expr(expr);

        let static_ty = make_static(&item.raw.ty);

        quote! {
            pub mod #ident {
                use super::*;
                pub const __NAME: &str = stringify!(#ident);
                pub const __TY: &str = #ty;
                pub const __EXPR: &str = #expr_str;
                #[inline]
                pub fn value() -> #static_ty {
                    #expr
                }
                pub const DYN_STORY_CONST: narrative::story::DynStoryConst = narrative::story::DynStoryConst::new(
                    __NAME,
                    __TY,
                    __EXPR,
                    || narrative::value::BoxedValue::new(value()),
                    || narrative::value::BoxedValue::new(StoryConst::#ident)
                );
            }
        }
    });

    let name_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote!(Self::#ident => story_consts::#ident::__NAME,)
        })
        .collect::<MatchArms>();

    let ty_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote!(Self::#ident => story_consts::#ident::__TY,)
        })
        .collect::<MatchArms>();

    let expr_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote!(Self::#ident => story_consts::#ident::__EXPR,)
        })
        .collect::<MatchArms>();

    let value_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote!(Self::#ident => ConstValue::#ident(story_consts::#ident::value()),)
        })
        .collect::<MatchArms>()
        .cast_as(quote!(narrative::value::BoxedValue));

    let impl_body = quote! {
        #[inline]
        fn name(&self) -> &'static str {
            #name_arms
        }
        #[inline]
        fn ty(&self) -> &'static str {
            #ty_arms
        }
        #[inline]
        fn expr(&self) -> &'static str {
            #expr_arms
        }
        #[inline]
        fn value(&self) -> impl narrative::value::Value {
            #value_arms
        }
    };

    quote! {
        #[derive(Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum StoryConst {
            #(#const_names),*
        }

        #[derive(Clone, narrative::serde::Serialize)]
        #[allow(non_camel_case_types)]
        enum ConstValue {
            #(#const_variants),*
        }

        impl std::fmt::Debug for ConstValue {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #const_value_debug_arms
            }
        }

        mod story_consts {
            use super::*;
            #(#const_mod_defs)*
        }

        impl StoryConst {
            pub fn to_dyn(&self) -> narrative::story::DynStoryConst {
                match self {
                    #(Self::#const_names => story_consts::#const_names::DYN_STORY_CONST,)*
                }
            }
        }

        impl narrative::story::StoryConst for StoryConst {
            #impl_body
        }

        impl std::fmt::Debug for StoryConst {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                use narrative::story::StoryConst;
                write!(f, "{name}: {ty} = {expr}", name = self.name(), ty = self.ty(), expr = self.expr())
            }
        }

        impl narrative::serde::Serialize for StoryConst {
            #[inline]
            fn serialize<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                use narrative::story::StoryConst;
                use narrative::serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("name", self.name())?;
                map.serialize_entry("ty", self.ty())?;
                map.serialize_entry("expr", self.expr())?;
                map.serialize_entry("value", &self.value())?;
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use syn::parse_quote;

    #[test]
    fn empty() {
        let story = parse_quote! {
            trait User {
            }
        };

        let actual = generate(&story);

        assert_eq!(
            actual.to_string(),
            quote! {
                /// This is placeholder and never be constructed.
                pub type StoryConst = narrative::story::DynStoryConst;
            }
            .to_string()
        );
    }

    #[test]
    fn consts() {
        let story = parse_quote! {
            trait User {
                const NUMBER: u32 = 42;
            }
        };

        let actual = generate(&story);

        assert_eq!(
            actual.to_string(),
            quote! {
                #[derive(Clone, Copy)]
                #[allow(non_camel_case_types)]
                pub enum StoryConst {
                    NUMBER
                }

                #[derive(Clone, narrative::serde::Serialize)]
                #[allow(non_camel_case_types)]
                enum ConstValue {
                    NUMBER(u32)
                }

                impl std::fmt::Debug for ConstValue {
                    #[inline]
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        match self {
                            Self::NUMBER(value) => value.fmt(f),
                        }
                    }
                }

                mod story_consts {
                    use super::*;
                    pub mod NUMBER {
                        use super::*;
                        pub const __NAME: &str = stringify!(NUMBER);
                        pub const __TY: &str = "u32";
                        pub const __EXPR: &str = "42";
                        #[inline]
                        pub fn value() -> u32 {
                            42
                        }
                        pub const DYN_STORY_CONST: narrative::story::DynStoryConst = narrative::story::DynStoryConst::new(
                            __NAME,
                            __TY,
                            __EXPR,
                            || narrative::value::BoxedValue::new(value()),
                            || narrative::value::BoxedValue::new(StoryConst::NUMBER)
                        );
                    }
                }

                impl StoryConst {
                    pub fn to_dyn(&self) -> narrative::story::DynStoryConst {
                        match self {
                            Self::NUMBER => story_consts::NUMBER::DYN_STORY_CONST,
                        }
                    }
                }

                impl narrative::story::StoryConst for StoryConst {
                    #[inline]
                    fn name(&self) -> &'static str {
                        match self {
                            Self::NUMBER => story_consts::NUMBER::__NAME,
                        }
                    }
                    #[inline]
                    fn ty(&self) -> &'static str {
                        match self {
                            Self::NUMBER => story_consts::NUMBER::__TY,
                        }
                    }
                    #[inline]
                    fn expr(&self) -> &'static str {
                        match self {
                            Self::NUMBER => story_consts::NUMBER::__EXPR,
                        }
                    }
                    #[inline]
                    fn value(&self) -> impl narrative::value::Value {
                        match self {
                            Self::NUMBER => ConstValue::NUMBER(story_consts::NUMBER::value()),
                        }
                    }
                }

                impl std::fmt::Debug for StoryConst {
                    #[inline]
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        use narrative::story::StoryConst;
                        write!(f, "{name}: {ty} = {expr}", name = self.name(), ty = self.ty(), expr = self.expr())
                    }
                }

                impl narrative::serde::Serialize for StoryConst {
                    #[inline]
                    fn serialize<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                        use narrative::story::StoryConst;
                        use narrative::serde::ser::SerializeMap;
                        let mut map = serializer.serialize_map(Some(4))?;
                        map.serialize_entry("name", self.name())?;
                        map.serialize_entry("ty", self.ty())?;
                        map.serialize_entry("expr", self.expr())?;
                        map.serialize_entry("value", &self.value())?;
                        map.end()
                    }
                }
            }
            .to_string()
        );
    }
}
