// important! We don't make step args types camel case, to support rich rename experience.
// To avoid name conflict, we use dedicated module for args.
// enum dispatched by step name and step arg name.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};

use crate::{
    item_story::{ItemStory, StoryStep},
    output::MatchArms,
};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let step_names: Vec<_> = story.steps().map(|step| &step.inner.sig.ident).collect();
    let step_enums = story.steps().map(generate_step_enum);
    let arg_impls = story.steps().map(|step| generate_arg_impl(story, step));
    let serde_impls = story.steps().map(generate_serialize_impl);
    let debug_impls = story.steps().map(generate_debug_impl);
    quote! {
        #[derive(Clone, Copy, narrative::serde::Serialize)]
        #[serde(transparent, crate = "narrative::serde")]
        pub struct StepArg(StepArgInner);

        #[derive(Clone, Copy, narrative::serde::Serialize)]
        #[allow(non_camel_case_types)]
        #[serde(untagged, crate = "narrative::serde")]
        enum StepArgInner {
            #(#step_names(args::#step_names)),*
        }

        impl narrative::step::StepArg for StepArg {
            #[inline]
            fn name(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.name()),*
                }
            }
            #[inline]
            fn ty(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.ty()),*
                }
            }
            #[inline]
            fn expr(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.expr()),*
                }
            }
            #[inline]
            fn debug_value(&self) -> String {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.debug_value()),*
                }
            }
            #[inline]
            fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.serialize_value(serializer)),*
                }
            }
        }

        impl std::fmt::Debug for StepArg {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.fmt(f)),*
                }
            }
        }

        mod args {
            #(#step_enums)*
            #(#arg_impls)*
            #(#serde_impls)*
            #(#debug_impls)*
        }
    }
}

fn generate_debug_impl(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    quote! {
        impl std::fmt::Debug for #step_ident {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{name}: {ty} = {expr}", name = self.name(), ty = self.ty(), expr = self.expr())
            }
        }
    }
}

fn generate_step_enum(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    let variants = step.fn_args().map(|(ident, _)| ident);
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        pub(super) enum #step_ident {
            #(#variants,)*
        }
    }
}

fn generate_serialize_impl(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    quote! {
        impl narrative::serde::Serialize for #step_ident {
            #[inline]
            fn serialize<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
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

fn generate_arg_impl(story: &ItemStory, step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    let name_arms = step
        .fn_args()
        .map(|(ident, _)| {
            quote! {
                Self::#ident => stringify!(#ident),
            }
        })
        .collect::<MatchArms>();
    let ty_arms = step
        .fn_args()
        .map(|(ident, ty)| {
            quote! {
                Self::#ident => stringify!(#ty),
            }
        })
        .collect::<MatchArms>();
    let arms = step.fn_args().map(|(ident, ty)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        (
            quote![Self::#ident => stringify!(#expr),],
            quote![Self::#ident => {
                let #ident: #ty = #expr;
                format!("{:?}", #ident)
            }],
            quote![Self::#ident => {
                let #ident: #ty = #expr;
                #ident.serialize(serializer)
            }],
        )
    }).collect::<Vec<_>>();
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
        impl #step_ident {
            #[inline]
            pub(super) fn name(&self) -> &'static str {
                #name_arms
            }
            #[inline]
            pub(super) fn ty(&self) -> &'static str {
                #ty_arms
            }
            #[inline]
            pub(super) fn expr(&self) -> &'static str {
                #expr_arms
            }
            #[inline]
            pub(super) fn debug_value(&self) -> String {
                #debug_arms
            }
            #[inline]
            pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                #[allow(unused_imports)]
                use narrative::serde::Serialize;
                #[allow(unnecessary_cast)]
                #serialize_arms
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use syn::parse2;

    #[test]
    fn simple() {
        let step = quote! {
            #[step("Step 1")]
            fn my_step1();
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    unreachable!()
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    unreachable!()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn use_step_attr_args() {
        let step = quote! {
            #[step("Step 1: {name}", name = "ryo")]
            fn my_step1(name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::name => stringify!(name),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::name => stringify!(&str),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::name => stringify!("ryo"),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::name => {
                            let name: &str = "ryo";
                            format!("{:?}", name)
                        }
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::name => {
                            let name: &str = "ryo";
                            name.serialize(serializer)
                        }
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    /// Find global assignments
    fn test_global_assignments() {
        let step = quote! {
            #[step("Step 1")]
            fn my_step1(id: UserId, name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                const id: UserId = UserId::new();
                const name: &str = "Alice";
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(id),
                        Self::name => stringify!(name),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId),
                        Self::name => stringify!(&str),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId::new()),
                        Self::name => stringify!("Alice"),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::id => {
                            let id: UserId = UserId::new();
                            format!("{:?}", id)
                        }
                        Self::name => {
                            let name: &str = "Alice";
                            format!("{:?}", name)
                        }
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::id => {
                            let id: UserId = UserId::new();
                            id.serialize(serializer)
                        }
                        Self::name => {
                            let name: &str = "Alice";
                            name.serialize(serializer)
                        }
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    /// Find local assignments first
    fn test_local_assignments() {
        let step = quote! {
            #[step("Step 1", name = "Bob")]
            fn my_step1(id: UserId, name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                const id: UserId = UserId::new();
                const name: &str = "Alice";
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(id),
                        Self::name => stringify!(name),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId),
                        Self::name => stringify!(&str),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId::new()),
                        Self::name => stringify!("Bob"),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::id => {
                            let id: UserId = UserId::new();
                            format!("{:?}", id)
                        }
                        Self::name => {
                            let name: &str = "Bob";
                            format!("{:?}", name)
                        }
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::id => {
                            let id: UserId = UserId::new();
                            id.serialize(serializer)
                        }
                        Self::name => {
                            let name: &str = "Bob";
                            name.serialize(serializer)
                        }
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
