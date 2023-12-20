// important! We don't make step args types camel case, to support rich rename experience.
// To avoid name conflict, we use dedicated module for args.
// enum dispatched by step name and step arg name.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};

use crate::item_story::{ItemStory, StoryStep};

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
                write!(f, "{name}: {ty} = {debug}", name = self.name(), ty = self.ty(), debug = self.debug_value())
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
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("name", self.name())?;
                map.serialize_entry("ty", self.ty())?;
                map.serialize_entry("debug", &self.debug_value())?;
                map.end()
            }
        }
    }
}

fn generate_arg_impl(story: &ItemStory, step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    let name_arms = step.fn_args().map(|(ident, _)| {
        quote! {
            Self::#ident => stringify!(#ident),
        }
    });
    let ty_arms = step.fn_args().map(|(ident, ty)| {
        quote! {
            Self::#ident => stringify!(#ty),
        }
    });
    let expr_arms = step.fn_args().map(|(ident, _)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        quote! {
            Self::#ident => stringify!(#expr),
        }
    });
    let debug_arms = step.fn_args().map(|(ident, _)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        quote! {
            Self::#ident => format!("{:?}", #expr),
        }
    });
    let serialize_arms = step.fn_args().map(|(ident, ty)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        quote! {
            Self::#ident => (#expr as #ty).serialize(serializer),
        }
    });
    quote! {
        impl #step_ident {
            #[inline]
            pub(super) fn name(&self) -> &'static str {
                match self {
                    #(#name_arms)*
                    _ => todo!(),
                }
            }
            #[inline]
            pub(super) fn ty(&self) -> &'static str {
                match self {
                    #(#ty_arms)*
                    _ => todo!(),
                }
            }
            #[inline]
            pub(super) fn expr(&self) -> &'static str {
                match self {
                    #(#expr_arms)*
                    _ => todo!(),
                }
            }
            #[inline]
            pub(super) fn debug_value(&self) -> String {
                match self {
                    #(#debug_arms)*
                    _ => todo!(),
                }
            }
            #[inline]
            pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                #[allow(unused_imports)]
                use narrative::serde::Serialize;
                #[allow(unnecessary_cast)]
                match self {
                    #(#serialize_arms)*
                    _ => todo!(),
                }
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
                    match self {
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        _ => todo!(),
                    }
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
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::name => stringify!(&str),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::name => stringify!("ryo"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::name => format!("{:?}", "ryo"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::name => ("ryo" as &str).serialize(serializer),
                        _ => todo!(),
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
                let id = UserId::new();
                let name = "Alice";
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
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId),
                        Self::name => stringify!(&str),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId::new()),
                        Self::name => stringify!("Alice"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::id => format!("{:?}", UserId::new()),
                        Self::name => format!("{:?}", "Alice"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::id => (UserId::new() as UserId).serialize(serializer),
                        Self::name => ("Alice" as &str).serialize(serializer),
                        _ => todo!(),
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
                let id = UserId::new();
                let name = "Alice";
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
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId),
                        Self::name => stringify!(&str),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => stringify!(UserId::new()),
                        Self::name => stringify!("Bob"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn debug_value(&self) -> String {
                    match self {
                        Self::id => format!("{:?}", UserId::new()),
                        Self::name => format!("{:?}", "Bob"),
                        _ => todo!(),
                    }
                }
                #[inline]
                pub(super) fn serialize_value<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    #[allow(unused_imports)]
                    use narrative::serde::Serialize;
                    #[allow(unnecessary_cast)]
                    match self {
                        Self::id => (UserId::new() as UserId).serialize(serializer),
                        Self::name => ("Bob" as &str).serialize(serializer),
                        _ => todo!(),
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
