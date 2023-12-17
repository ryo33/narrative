// These methods are separated from the real story trait not to provide default
// implementations for them. It supresses "Implement default members for trait" action.
// Theorically, instead of this, we can use a sealed default implementation, but it's not friendly
// interface for narrative.
// It's a wrapper of the story context

use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, story_attr_syntax::StoryAttr, Asyncness};

pub(crate) fn generate(attr: &StoryAttr, input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = &input.ident;
    let story_title = &attr.title;
    quote! {
        pub trait StoryExt: Sized {
            type Error: std::error::Error;
            /// Returns the title of the story.
            fn story_title() -> String;
            /// Returns the identifier of the story.
            fn story_ident() -> &'static str;
            /// Returns the steps of the story.
            fn steps() -> Steps<Self, Self::Error>;
            /// Run all steps in the story. It's a shortcut for iterating over the steps.
            fn run_all(self) -> Result<(), narrative::RunAllError<StepId, Self::Error>>;
            /// Get the context of this story.
            fn context() -> StoryContext<Self>;
        }
        impl <T: #ident> StoryExt for T {
            type Error = T::Error;
            fn story_title() -> String {
                #story_title.to_string()
            }
            fn story_ident() -> &'static str {
                stringify!(#ident)
            }
            fn steps() -> Steps<Self, Self::Error> {
                self.context().steps()
            }
            fn run_all(self) -> Result<(), narrative::RunAllError<StepId, Self::Error>> {
                for step in Self::steps() {
                    if let Err(e) = step.run(&mut self) {
                        return Err(narrative::RunAllError {
                            step_id: step.id,
                            error: e,
                        });
                    }
                }
                Ok(())
            }
            fn context(&self) -> StoryContext<Self> {
                StoryContext {
                    phantom: std::marker::PhantomData,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate_sync() {
        let attr = syn::parse_quote! {
            "User story title"
        };
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&attr, &story_syntax, Asyncness::Sync);
        let expected = quote! {
            pub trait StoryExt: Sized {
                type Error: std::error::Error;
                // This is not &str for future extensibility.
                /// Returns the title of the story.
                fn story_title() -> String;
                /// Returns the identifier of the story.
                fn story_ident() -> &'static str;
                /// Returns the steps of the story.
                fn steps() -> Steps<Self, Self::Error>;
                /// Run all steps in the story. It's a shortcut for iterating over the steps.
                fn run_all(self) -> Result<(), narrative::RunAllError<StepId, Self::Error>>;
                /// Get the context of this story.
                fn context() -> StoryContext<Self>;
            }
            impl <T: UserStory> StoryExt for T {
                type Error = T::Error;
                fn story_title() -> String {
                    "User story title".to_string()
                }
                fn story_ident() -> &'static str {
                    stringify!(UserStory)
                }
                fn steps() -> Steps<Self, Self::Error> {
                    self.context().steps()
                }
                fn run_all(self) -> Result<(), narrative::RunAllError<StepId, Self::Error>> {
                    for step in Self::steps() {
                        if let Err(e) = step.run(&mut self) {
                            return Err(narrative::RunAllError {
                                step_id: step.id,
                                error: e,
                            });
                        }
                    }
                    Ok(())
                }
                fn context(&self) -> StoryContext<Self> {
                    StoryContext {
                        phantom: std::marker::PhantomData,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
