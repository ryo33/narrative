use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    item_story::{ItemStory, StoryItem},
    Asyncness,
};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let ident = &input.ident;
    let steps = input
        .items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| match item {
            StoryItem::Step(step) => {
                let step_name = &step.inner.sig.ident;
                let step_text = &step.attr.text;
                let args = step.inner.sig.inputs.iter().filter_map(|arg| {
                    if let syn::FnArg::Typed(arg) = arg {
                        let name = &arg.pat;
                        let ty = &arg.ty;
                        // TODO: get value from attr args or global assignments
                        let value = quote! { aaa };
                        Some(quote! {
                            Arg {
                                name: stringify!(#name),
                                ty: stringify!(#ty),
                                value_fn: || {
                                    format!("{:?}", #value)
                                },
                            }
                        })
                    } else {
                        None
                    }
                });
                Some(quote! {
                    Step {
                        index: #idx,
                        step_fn: |state| {
                            self.#step_name(Default::default(), state)
                        },
                        step_text: || {
                            // TODO:format
                            format!(#step_text)
                        },
                        args: Args::new(&[
                            #(#args),*
                        ]),
                    },
                })
            }
            _ => None,
        });
    quote! {
        #[derive(Default)]
        pub struct StoryContext<T: #ident> {
            phantom: std::marker::PhantomData<T>,
        }
        impl <T: #ident> StoryContext<T> {
            pub fn steps(&self) -> Self {
                static STEPS: &[Step<T, T::Error>] = &[];
                Self {
                    steps: Steps {
                        steps: STEPS,
                    },
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
    fn test_generate() {
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&story_syntax, Asyncness::Sync);
        let expected = quote! {
            #[derive(Default)]
            pub struct StoryContext<T: UserStory> {
                phantom: std::marker::PhantomData<T>,
            }
            // &self in these methods are not necessary but it's for future extensibility and friendly API.
            impl <T: UserStory> StoryContext<T> {
                pub fn steps<T: UserStory>(&self) -> Steps<T, T::Error> {
                    static STEPS: &[Step<T, T::Error>] = &[
                        Step {
                            index: 0,
                            step_fn: |state| {
                                self.step1(Default::default(), state)
                            },
                            step_text: || {
                                "step1".to_string()
                            },
                            args: Args::new(&[]),
                        },
                        Step {
                            index: 1,
                            step_fn: |state| {
                                self.step2(Default::default(), state)
                            },
                            step_text: || {
                                let name = "ryo";
                                format!("step2: {name}", name)
                            },
                            args: Args::new(&[
                                Arg {
                                    name: "name",
                                    ty: "&str",
                                    value_fn: || {
                                        "ryo".to_string()
                                    },
                                },
                            ]),
                        },
                    ];
                    Self {
                        steps: Steps {
                            steps: STEPS,
                        },
                    }
                }
                pub fn step1(&self, story: &mut T) -> Result<(), T::Error> {
                    story.step1()
                }
                pub fn step2(&self, story: &mut T) -> Result<(), T::Error> {
                    let name = "ryo";
                    story.step2(name)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
