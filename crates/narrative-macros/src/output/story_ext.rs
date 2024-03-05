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
        pub trait #ext_ident: Sized {
            type Error: std::error::Error;
            /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
            #sig -> Result<(), Self::Error>;
        }
        impl <T: #ident> #ext_ident for T {
            type Error = T::Error;
            #[inline]
            #sig -> Result<(), Self::Error> {
                use narrative::step::Step;
                #use_run
                for step in narrative::story::StoryContext::steps(&StoryContext::default()) {
                    #step_run?;
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
                /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
                fn run_all(&mut self) -> Result<(), Self::Error>;
            }
            impl <T: UserStory> StoryExt for T {
                type Error = T::Error;
                #[inline]
                fn run_all(&mut self) -> Result<(), Self::Error> {
                    use narrative::step::Step;
                    use narrative::step::Run;
                    for step in narrative::story::StoryContext::steps(&StoryContext::default()) {
                        step.run(self)?;
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
            pub trait AsyncStoryExt: Sized {
                type Error: std::error::Error;
                /// Run all steps of the story. This is a shortcut to iterate all steps and run them.
                async fn run_all_async(&mut self) -> Result<(), Self::Error>;
            }
            impl <T: AsyncUserStory> AsyncStoryExt for T {
                type Error = T::Error;
                #[inline]
                async fn run_all_async(&mut self) -> Result<(), Self::Error> {
                    use narrative::step::Step;
                    use narrative::step::RunAsync;
                    for step in narrative::story::StoryContext::steps(&StoryContext::default()) {
                        step.run_async(self).await?;
                    }
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
