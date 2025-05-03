// &self in these methods are not necessary but it's for future extensibility and friendly API.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    item_story::{story_const::StoryConst, ItemStory},
    story_attr_syntax::StoryAttr,
};

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
    let consts = input.consts().map(|item| &item.raw.ident);
    let consts_defs = input.consts().map(
        |StoryConst {
             raw,
             default: (eq, default),
         }| {
            let ident = &raw.ident;
            let ty = &raw.ty;
            Some(quote! {
                pub const #ident: #ty #eq #default;
            })
        },
    );
    let steps_len = input.steps().count();
    let const_len = input.consts().count();
    let dyn_consts = if const_len == 0 {
        quote!(Box::new(std::iter::empty()))
    } else {
        quote!(Box::new(__CONSTS.into_iter().map(|c| c.to_dyn())))
    };
    let dyn_steps = if steps_len == 0 {
        quote!(Box::new(std::iter::empty()))
    } else {
        quote!(Box::new(__STEPS.into_iter().map(|s| s.to_dyn())))
    };
    quote! {
        #[derive(Default, Clone, Copy)]
        pub struct StoryContext;
        pub const __STORY_TITLE: &str = #title;
        pub const __STORY_ID: &str = stringify!(#ident);
        pub const __STEPS: [Step; #steps_len] = [#(Step::#step_names),*];
        pub const __CONSTS: [StoryConst; #const_len] = [#(StoryConst::#consts),*];
        impl StoryContext {
            #(#consts_defs)*
            #(#steps)*

            pub fn to_dyn(&self) -> narrative::story::DynStoryContext {
                narrative::story::DynStoryContext::new(
                    __STORY_TITLE,
                    __STORY_ID,
                    || #dyn_consts,
                    || #dyn_steps,
                )
            }
        }
        impl narrative::story::StoryContext for StoryContext {
            type Step = Step;

            #[inline]
            fn story_title(&self) -> String {
                __STORY_TITLE.to_string()
            }
            #[inline]
            fn story_id(&self) -> &'static str {
                __STORY_ID
            }
            #[inline]
            fn steps(&self) -> impl Iterator<Item = Self::Step> + 'static + Send {
                __STEPS.into_iter()
            }
            #[inline]
            fn consts(&self) -> impl Iterator<Item = impl narrative::story::StoryConst + 'static> + 'static
            {
                __CONSTS.into_iter()
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
                const NAME: &str = "Ryo";
                const AGE: u32 = 20;

                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&attr, &story_syntax);
        let expected = quote! {
            #[derive(Default, Clone, Copy)]
            pub struct StoryContext;
            pub const __STORY_TITLE: &str = "Story Title";
            pub const __STORY_ID: &str = stringify!(UserStory);
            pub const __STEPS: [Step; 2usize] = [Step::step1, Step::step2];
            pub const __CONSTS: [StoryConst; 2usize] = [StoryConst::NAME, StoryConst::AGE];
            impl StoryContext {
                pub const NAME: &str = "Ryo";
                pub const AGE: u32 = 20;

                #[inline]
                pub fn step1(&self) -> Step {
                    Step::step1
                }
                #[inline]
                pub fn step2(&self) -> Step {
                    Step::step2
                }

                pub fn to_dyn(&self) -> narrative::story::DynStoryContext {
                    narrative::story::DynStoryContext::new(
                        __STORY_TITLE,
                        __STORY_ID,
                        || Box::new(__CONSTS.into_iter().map(|c| c.to_dyn())),
                        || Box::new(__STEPS.into_iter().map(|s| s.to_dyn())),
                    )
                }
            }
            impl narrative::story::StoryContext for StoryContext {
                type Step = Step;

                #[inline]
                fn story_title(&self) -> String {
                    __STORY_TITLE.to_string()
                }
                #[inline]
                fn story_id(&self) -> &'static str {
                    __STORY_ID
                }
                #[inline]
                fn steps(&self) -> impl Iterator<Item = Self::Step> + 'static + Send {
                    __STEPS.into_iter()
                }
                #[inline]
                fn consts(&self) -> impl Iterator<Item = impl narrative::story::StoryConst + 'static> + 'static
                {
                    __CONSTS.into_iter()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
