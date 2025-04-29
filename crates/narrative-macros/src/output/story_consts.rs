// important! We don't make story const types camel case, to support rich rename experience on rust-analyzer.
// To avoid name conflict, we use dedicated module for consts.
// enum dispatched by const name

use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, output::MatchArms};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let const_names = story.consts().map(|item| &item.raw.ident);
    let const_variants = story.consts().map(|item| {
        let ident = &item.raw.ident;
        let ty = &item.raw.ty;
        quote!(#ident(#ty))
    });
    let name_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            quote! {
                StoryConst::#ident => stringify!(#ident),
            }
        })
        .collect::<MatchArms>();
    let ty_arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            let ty = &item.raw.ty;
            quote! {
                StoryConst::#ident => stringify!(#ty),
            }
        })
        .collect::<MatchArms>();
    let arms = story
        .consts()
        .map(|item| {
            let ident = &item.raw.ident;
            let ty = &item.raw.ty;
            let expr = &item.default.1;
            (
                quote![StoryConst::#ident => stringify!(#expr),],
                quote![StoryConst::#ident => {
                    let #ident: #ty = #expr;
                    format!("{:?}", #ident)
                }],
                quote![StoryConst::#ident => {
                    let #ident: #ty = #expr;
                    ConstValue::#ident(#ident)
                }],
            )
        })
        .collect::<Vec<_>>();
    let expr_arms = arms.iter().map(|(expr, _, _)| expr).collect::<MatchArms>();
    let debug_arms = arms
        .iter()
        .map(|(_, debug, _)| debug)
        .collect::<MatchArms>();
    let serialize_arms = arms
        .iter()
        .map(|(_, _, serialize)| serialize)
        .collect::<MatchArms>()
        .cast_as(quote!(&str));

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

        impl narrative::story::StoryConst for StoryConst {
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
            fn debug_value(&self) -> String {
                #debug_arms
            }
            #[inline]
            fn serialize_value(&self) -> impl serde::Serialize + 'static {
                #serialize_arms
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
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("name", self.name())?;
                map.serialize_entry("ty", self.ty())?;
                map.serialize_entry("expr", &self.expr())?;
                map.serialize_entry("debug", &self.debug_value())?;
                map.serialize_entry("value", self)?;
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
                #[derive(Clone, Copy)]
                #[allow(non_camel_case_types)]
                pub enum StoryConst {}

                #[derive(Clone, narrative::serde::Serialize)]
                #[allow(non_camel_case_types)]
                enum ConstValue {}

                impl narrative::story::StoryConst for StoryConst {
                    #[inline]
                    fn name(&self) -> &'static str {
                        unreachable!()
                    }
                    #[inline]
                    fn ty(&self) -> &'static str {
                        unreachable!()
                    }
                    #[inline]
                    fn expr(&self) -> &'static str {
                        unreachable!()
                    }
                    #[inline]
                    fn debug_value(&self) -> String {
                        unreachable!()
                    }
                    #[inline]
                    fn serialize_value(&self) -> impl serde::Serialize + 'static {
                        #[allow(unreachable_code)]
                        {
                            unreachable!() as &str
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
                        let mut map = serializer.serialize_map(Some(5))?;
                        map.serialize_entry("name", self.name())?;
                        map.serialize_entry("ty", self.ty())?;
                        map.serialize_entry("expr", &self.expr())?;
                        map.serialize_entry("debug", &self.debug_value())?;
                        map.serialize_entry("value", self)?;
                        map.end()
                    }
                }
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

                impl narrative::story::StoryConst for StoryConst {
                    #[inline]
                    fn name(&self) -> &'static str {
                        match self {
                            StoryConst::NUMBER => stringify!(NUMBER),
                        }
                    }
                    #[inline]
                    fn ty(&self) -> &'static str {
                        match self {
                            StoryConst::NUMBER => stringify!(u32),
                        }
                    }
                    #[inline]
                    fn expr(&self) -> &'static str {
                        match self {
                            StoryConst::NUMBER => stringify!(42),
                        }
                    }
                    #[inline]
                    fn debug_value(&self) -> String {
                        match self {
                            StoryConst::NUMBER => {
                                let NUMBER: u32 = 42;
                                format!("{:?}", NUMBER)
                            }
                        }
                    }
                    #[inline]
                    fn serialize_value(&self) -> impl serde::Serialize + 'static {
                        match self {
                            StoryConst::NUMBER => {
                                let NUMBER: u32 = 42;
                                ConstValue::NUMBER(NUMBER)
                            }
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
                        let mut map = serializer.serialize_map(Some(5))?;
                        map.serialize_entry("name", self.name())?;
                        map.serialize_entry("ty", self.ty())?;
                        map.serialize_entry("expr", &self.expr())?;
                        map.serialize_entry("debug", &self.debug_value())?;
                        map.serialize_entry("value", self)?;
                        map.end()
                    }
                }
            }
            .to_string()
        );
    }
}
