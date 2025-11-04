use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

use crate::{
    collect_format_args,
    step_attr_syntax::{StepAttr, StoryType},
};

pub struct StoryStep {
    pub attr: StepAttr,
    pub inner: syn::TraitItemFn,
}

impl Parse for StoryStep {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let inner = input.parse()?;
        Ok(Self { attr, inner })
    }
}

impl ToTokens for StoryStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attr.to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}

impl StoryStep {
    pub(crate) fn attr_args(&self) -> impl Iterator<Item = (&syn::Ident, &syn::Expr)> {
        self.attr.args.iter().map(|arg| (&arg.ident, &arg.value))
    }

    /// This ignores patterns.
    pub(crate) fn fn_args(&self) -> impl Iterator<Item = (&syn::Ident, &syn::Type)> {
        self.inner
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(pat_type) => {
                    if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        Some((&pat_ident.ident, pat_type.ty.as_ref()))
                    } else {
                        None
                    }
                }
                syn::FnArg::Receiver(_) => None,
            })
    }

    pub(crate) fn find_attr_arg<'a>(&'a self, ident: &'a syn::Ident) -> Option<&'a syn::Expr> {
        self.attr_args().find_map(move |(arg_ident, arg_value)| {
            if arg_ident == ident {
                Some(arg_value)
            } else {
                None
            }
        })
    }

    pub(crate) fn extract_format_args(&self) -> Vec<String> {
        collect_format_args(&self.attr.text)
    }

    pub(crate) fn has_sub_story(&self) -> bool {
        self.attr.story_type.is_some()
    }

    /// Gets the path to the sub-story type if this is a sub-story step
    pub(crate) fn sub_story_path(&self) -> Option<&StoryType> {
        self.attr.story_type.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_step() {
        let input = quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = syn::parse2::<StoryStep>(input).unwrap();
        assert_eq!(actual.attr.text.value(), "Step 1".to_string());
        assert_eq!(actual.inner.sig.ident.to_string(), "step1".to_string());
        assert!(actual.sub_story_path().is_none());
    }

    #[test]
    fn parse_sub_story_step() {
        let input = quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub();
        };
        let actual = syn::parse2::<StoryStep>(input).unwrap();
        assert_eq!(actual.attr.text.value(), "do sub story".to_string());
        assert_eq!(
            actual.inner.sig.ident.to_string(),
            "step_with_sub".to_string()
        );
        assert!(actual.sub_story_path().unwrap().path.is_ident("SubStory"));
    }

    #[test]
    fn to_tokens() {
        let input = quote! {
            #[step("Step 1")]
            fn step1();
        };
        let actual = syn::parse2::<StoryStep>(input).unwrap();
        let actual = quote! {
            #actual
        };
        let expected = quote! {
            #[step("Step 1")]
            fn step1();
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn to_tokens_sub_story() {
        let input = quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub();
        };
        let actual = syn::parse2::<StoryStep>(input).unwrap();
        let actual = quote! {
            #actual
        };
        let expected = quote! {
            #[step(story: SubStory, "do sub story")]
            fn step_with_sub();
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
