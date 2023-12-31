// &self in these methods are not necessary but it's for future extensibility and friendly API.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{item_story::ItemStory, story_attr_syntax::StoryAttr};

pub(crate) fn generate(attr: &StoryAttr, input: &ItemStory) -> TokenStream {
    let title = &attr.title;
    let ident = &input.ident;
    let steps = input.steps().map(|step| {
        let step_name = &step.inner.sig.ident;
        quote! {
            #[inline]
            pub fn #step_name(&self) -> Step {
                Step::#step_name
            }
        }
    });
    let step_names = input.steps().map(|step| &step.inner.sig.ident);
    quote! {
        #[derive(Default)]
        pub struct StoryContext;
        impl StoryContext {
            #(#steps)*
        }
        impl narrative::story::StoryContext for StoryContext {
            type Step = Step;
            type StepIter = std::slice::Iter<'static, Self::Step>;
            #[inline]
            fn story_title(&self) -> String {
                #title.to_string()
            }
            #[inline]
            fn story_id(&self) -> &'static str {
                stringify!(#ident)
            }
            #[inline]
            fn steps(&self) -> Self::StepIter {
                [#(Step::#step_names),*].iter()
            }
        }
    }
}

pub(crate) fn generate_ext(input: &ItemStory) -> TokenStream {
    let ident = &input.ident;
    let async_ident = format_ident!("Async{}", input.ident);
    quote! {
        pub trait ContextExt {
            fn context() -> StoryContext;
            fn get_context(&self) -> StoryContext;
        }
        pub trait AsyncContextExt {
            fn context() -> StoryContext;
            fn get_context(&self) -> StoryContext;
        }
        impl <T: #ident> ContextExt for T {
            #[inline]
            fn context() -> StoryContext {
                StoryContext::default()
            }
            #[inline]
            fn get_context(&self) -> StoryContext {
                StoryContext::default()
            }
        }
        impl <T: #async_ident> AsyncContextExt for T {
            #[inline]
            fn context() -> StoryContext {
                StoryContext::default()
            }
            #[inline]
            fn get_context(&self) -> StoryContext {
                StoryContext::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate() {
        let attr = syn::parse_quote! {
            "Story Title"
        };
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&attr, &story_syntax);
        let expected = quote! {
            #[derive(Default)]
            pub struct StoryContext;
            impl StoryContext {
                #[inline]
                pub fn step1(&self) -> Step {
                    Step::step1
                }
                #[inline]
                pub fn step2(&self) -> Step {
                    Step::step2
                }
            }
            impl narrative::story::StoryContext for StoryContext {
                type Step = Step;
                type StepIter = std::slice::Iter<'static, Self::Step>;
                #[inline]
                fn story_title(&self) -> String {
                    "Story Title".to_string()
                }
                #[inline]
                fn story_id(&self) -> &'static str {
                    stringify!(UserStory)
                }
                #[inline]
                fn steps(&self) -> Self::StepIter {
                    [Step::step1, Step::step2].iter()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
