// important! We don't make story const types camel case, to support rich rename experience on rust-analyzer.
// To avoid name conflict, we use dedicated module for consts.
// enum dispatched by const name

use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, output::MatchArms};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let const_names = story.consts().map(|item| &item.raw.ident);
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
                quote![StoryConst::#ident => stringify!(#expr)],
                quote![StoryConst::#ident => format!("{:?}", #expr)],
                quote![StoryConst::#ident => (#expr as #ty).serialize(serializer)],
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
        .collect::<MatchArms>();
    quote! {
        #[derive(Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum StoryConst {
            #(#const_names),*
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
            fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                #[allow(unused_imports)]
                use narrative::serde::Serialize;
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
                    fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                        #[allow(unused_imports)]
                        use narrative::serde::Serialize;
                        unreachable!()
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
