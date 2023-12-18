// These methods are separated from the real story trait not to provide default
// implementations for them. It supresses "Implement default members for trait" action.
// Theorically, instead of this, we can use a sealed default implementation, but it's not friendly
// interface for narrative.
// It's a wrapper of the story context

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::{item_story::ItemStory, Asyncness};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = match asyncness {
        Asyncness::Sync => input.ident.clone(),
        Asyncness::Async => format_ident!("Async{}", input.ident),
    };
    let ext_ident = match asyncness {
        Asyncness::Sync => syn::Ident::new("StoryExt", Span::call_site()),
        Asyncness::Async => syn::Ident::new("AsyncStoryExt", Span::call_site()),
    };
    let async_trait = match asyncness {
        Asyncness::Sync => quote! {},
        Asyncness::Async => quote!(#[narrative::async_trait]),
    };
    let sig = match asyncness {
        Asyncness::Sync => quote! {fn run_all(&mut self)},
        Asyncness::Async => quote! {async fn run_all_async(&mut self)},
    };
    let step_run = match asyncness {
        Asyncness::Sync => quote! {step.run(self)},
        Asyncness::Async => quote! {step.run_async(self).await},
    };
    let use_run = match asyncness {
        Asyncness::Sync => quote! {use narrative::step::Run;},
        Asyncness::Async => quote! {use narrative::step::RunAsync;},
    };
    quote! {
        #async_trait
        pub trait #ext_ident: Sized {
            type Error: std::error::Error;
            /// Get the context of this story.
            fn context() -> StoryContext;
            /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
            #sig -> Result<(), narrative::RunAllError<StepId, Self::Error>>;
        }
        #async_trait
        impl <T: #ident> #ext_ident for T {
            type Error = T::Error;
            #[inline]
            fn context() -> StoryContext {
                Default::default()
            }
            #[inline]
            #sig -> Result<(), narrative::RunAllError<StepId, Self::Error>> {
                use narrative::step::Step;
                #use_run
                use narrative::story::StoryContext;
                for step in Self::context().steps() {
                    if let Err(e) = #step_run {
                        return Err(narrative::RunAllError {
                            step_id: step.id(),
                            error: e,
                        });
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_sync() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {}
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            pub trait StoryExt: Sized {
                type Error: std::error::Error;
                /// Get the context of this story.
                fn context() -> StoryContext<Self>;
                /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
                fn run_all(&mut self) -> Result<(), narrative::RunAllError<StepId, Self::Error>>;
            }
            impl <T: UserStory> StoryExt for T {
                type Error = T::Error;
                fn context() -> StoryContext<Self> {
                    Default::default()
                }
                fn run_all(&mut self) -> Result<(), narrative::RunAllError<StepId, Self::Error>> {
                    use narrative::step::Step;
                    use narrative::step::Run;
                    use narrative::story::StoryContext;
                    for step in Self::context().steps() {
                        if let Err(e) = step.run(self) {
                            return Err(narrative::RunAllError {
                                step_id: step.id(),
                                error: e,
                            });
                        }
                    }
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_async() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {}
        };
        let actual = generate(&story_syntax, Asyncness::Async);
        let expected = quote! {
            #[narrative::async_trait]
            pub trait AsyncStoryExt: Sized {
                type Error: std::error::Error;
                /// Get the context of this story.
                fn context() -> StoryContext;
                /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
                async fn run_all_async(&mut self) -> Result<(), narrative::RunAllError<StepId, Self::Error>>;
            }
            impl <T: AsyncUserStory> AsyncStoryExt for T {
                type Error = T::Error;
                fn context() -> StoryContext {
                    Default::default()
                }
                async fn run_all_async(&mut self) -> Result<(), narrative::RunAllError<StepId, Self::Error>> {
                    use narrative::step::Step;
                    use narrative::step::RunAsync;
                    use narrative::story::StoryContext;
                    for step in Self::context().steps() {
                        if let Err(e) = step.run_async(self).await {
                            return Err(narrative::RunAllError {
                                step_id: step.id(),
                                error: e,
                            });
                        }
                    }
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
