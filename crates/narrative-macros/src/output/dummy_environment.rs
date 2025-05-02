use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{item_story::ItemStory, output::step_fn, Asyncness};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = match asyncness {
        Asyncness::Sync => input.ident.clone(),
        Asyncness::Async => format_ident!("Async{}", input.ident),
    };
    let steps = input.steps().map(|step| {
        let step_fn = step_fn::generate(step, asyncness);
        let body = if step.has_sub_story() {
            quote!(Ok(
                narrative::environment::DummyEnvironment::<Self::Error>::default()
            ))
        } else {
            match asyncness {
                Asyncness::Sync => quote!(Ok(())),
                Asyncness::Async => quote!(async { Ok(()) }),
            }
        };
        quote! {
            #[inline]
            #[allow(clippy::manual_async_fn)]
            #step_fn {
                #body
            }
        }
    });

    quote! {
        #[allow(unused_variables)]
        impl<E: Send> #ident for narrative::environment::DummyEnvironment<E> {
            type Error = E;
            #(#steps)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            #[allow(unused_variables)]
            impl<E: Send> UserStory for narrative::environment::DummyEnvironment<E> {
                type Error = E;
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn step1(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn step2(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_async() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("Step 1")]
                fn step1();
                #[step("Step 2")]
                fn step2();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Async);
        let expected = quote! {
            #[allow(unused_variables)]
            impl<E: Send> AsyncUserStory for narrative::environment::DummyEnvironment<E> {
                type Error = E;
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn step1(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                    async { Ok(()) }
                }
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn step2(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                    async { Ok(()) }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_with_sub_story() {
        let story_syntax = syn::parse_quote! {
            trait StoryDef {
                #[step(story: OtherStory, "Sub Step 1")]
                fn sub_step_1();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            #[allow(unused_variables)]
            impl<E: Send> StoryDef for narrative::environment::DummyEnvironment<E> {
                type Error = E;
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn sub_step_1(&mut self) -> Result<impl OtherStory<Error = Self::Error>, Self::Error> {
                    Ok(narrative::environment::DummyEnvironment::<Self::Error>::default())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_async_with_sub_story() {
        let story_syntax = syn::parse_quote! {
            trait StoryDef {
                #[step(story: OtherStory, "Sub Step 1")]
                fn sub_step_1();
            }
        };
        let actual = generate(&story_syntax, Asyncness::Async);
        let expected = quote! {
            #[allow(unused_variables)]
            impl<E: Send> AsyncStoryDef for narrative::environment::DummyEnvironment<E> {
                type Error = E;
                #[inline]
                #[allow(clippy::manual_async_fn)]
                fn sub_step_1(&mut self) -> Result<impl AsyncOtherStory<Error = Self::Error>, Self::Error> {
                    Ok(narrative::environment::DummyEnvironment::<Self::Error>::default())
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
