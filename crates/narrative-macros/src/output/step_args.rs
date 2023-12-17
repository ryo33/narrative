// important! We don't make step args types camel case, to support rich rename experience.
// To mitigate the weird data names, we provide a wrapper type.
// To avoid name conflict, we use dedicated module for args.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};

use crate::item_story::{ItemStory, StoryStep};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let steps = story.steps().map(|step| generate_step(story, step));
    let debug_impls = story.steps().map(generate_debug_impl);
    quote! {
        #[derive(Clone, Copy, narrative::Serialize)]
        #[serde(transparent)]
        pub struct StepArg<T>(T);

        mod args {
            use super::*;
            #(#steps)*
            #(#debug_impls)*
        }
    }
}

fn generate_debug_impl(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    quote! {
        impl std::fmt::Debug for StepArg<#step_ident> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                use narrative::step::StepArg;
                write!(f, "{name}: {ty} = {debug}", name = self.name(), ty = self.ty(), debug = self.debug_value())
            }
        }
    }
}

fn generate_step(story: &ItemStory, step: &StoryStep) -> TokenStream {
    let story_ident = &step.inner.sig.ident;
    let variants = step.fn_arg_idents().map(|(ident, _)| ident);
    let name_arms = step.fn_arg_idents().map(|(ident, _)| {
        quote! {
            #story_ident::#ident => stringify!(#ident),
        }
    });
    let ty_arms = step.fn_arg_idents().map(|(ident, ty)| {
        quote! {
            #story_ident::#ident => stringify!(#ty),
        }
    });
    let debug_arms = step.fn_arg_idents().map(|(ident, _)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        quote! {
            #story_ident::#ident => format!("{:?}", #expr),
        }
    });
    let serialize_arms = step.fn_arg_idents().map(|(ident, ty)| {
        let expr = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident))
            .map(ToTokens::into_token_stream)
            .unwrap_or_else(
                || quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") },
            );
        quote! {
            #story_ident::#ident => (#expr as #ty).serialize(serializer),
        }
    });
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug)]
        pub enum #story_ident {
            #(#variants,)*
        }

        impl narrative::step::StepArg for StepArg<#story_ident> {
            fn name(&self) -> &'static str {
                match self.0 {
                    #(#name_arms)*
                }
            }
            fn ty(&self) -> &'static str {
                match self.0 {
                    #(#ty_arms)*
                }
            }
            fn debug_value(&self) -> String {
                match self.0 {
                    #(#debug_arms)*
                }
            }
            fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                use narrative::Serialize;
                match self.0 {
                    #(#serialize_arms)*
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
        let actual = generate_step(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug)]
            pub enum my_step1 {
            }

            impl narrative::step::StepArg for StepArg<my_step1> {
                fn name(&self) -> &'static str {
                    match self.0 {
                    }
                }
                fn ty(&self) -> &'static str {
                    match self.0 {
                    }
                }
                fn debug_value(&self) -> String {
                    match self.0 {
                    }
                }
                fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    use narrative::Serialize;
                    match self.0 {
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
        let actual = generate_step(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug)]
            pub enum my_step1 {
                name,
            }

            impl narrative::step::StepArg for StepArg<my_step1> {
                fn name(&self) -> &'static str {
                    match self.0 {
                        my_step1::name => stringify!(name),
                    }
                }
                fn ty(&self) -> &'static str {
                    match self.0 {
                        my_step1::name => stringify!(&str),
                    }
                }
                fn debug_value(&self) -> String {
                    match self.0 {
                        my_step1::name => format!("{:?}", "ryo"),
                    }
                }
                fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    use narrative::Serialize;
                    match self.0 {
                        my_step1::name => ("ryo" as &str).serialize(serializer),
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
        let actual = generate_step(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug)]
            pub enum my_step1 {
                id,
                name,
            }

            impl narrative::step::StepArg for StepArg<my_step1> {
                fn name(&self) -> &'static str {
                    match self.0 {
                        my_step1::id => stringify!(id),
                        my_step1::name => stringify!(name),
                    }
                }
                fn ty(&self) -> &'static str {
                    match self.0 {
                        my_step1::id => stringify!(UserId),
                        my_step1::name => stringify!(&str),
                    }
                }
                fn debug_value(&self) -> String {
                    match self.0 {
                        my_step1::id => format!("{:?}", UserId::new()),
                        my_step1::name => format!("{:?}", "Alice"),
                    }
                }
                fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    use narrative::Serialize;
                    match self.0 {
                        my_step1::id => (UserId::new() as UserId).serialize(serializer),
                        my_step1::name => ("Alice" as &str).serialize(serializer),
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
        let actual = generate_step(&story_syntax, &parse2(step).unwrap());
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug)]
            pub enum my_step1 {
                id,
                name,
            }

            impl narrative::step::StepArg for StepArg<my_step1> {
                fn name(&self) -> &'static str {
                    match self.0 {
                        my_step1::id => stringify!(id),
                        my_step1::name => stringify!(name),
                    }
                }
                fn ty(&self) -> &'static str {
                    match self.0 {
                        my_step1::id => stringify!(UserId),
                        my_step1::name => stringify!(&str),
                    }
                }
                fn debug_value(&self) -> String {
                    match self.0 {
                        my_step1::id => format!("{:?}", UserId::new()),
                        my_step1::name => format!("{:?}", "Bob"),
                    }
                }
                fn serialize_value<T: serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                    use narrative::Serialize;
                    match self.0 {
                        my_step1::id => (UserId::new() as UserId).serialize(serializer),
                        my_step1::name => ("Bob" as &str).serialize(serializer),
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_debug_impl() {
        let step = quote! {
            #[step("Step 1", name = "Bob")]
            fn my_step1(id: UserId, name: &str);
        };
        let actual = generate_debug_impl(&parse2(step).unwrap());
        let expected = quote! {
            impl std::fmt::Debug for StepArg<my_step1> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    use narrative::step::StepArg;
                    write!(f, "{name}: {ty} = {debug}", self.name(), self.ty(), self.debug_value())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
