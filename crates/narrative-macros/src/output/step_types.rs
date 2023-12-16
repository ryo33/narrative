use proc_macro2::TokenStream;
use quote::quote;

use crate::{item_story::ItemStory, Asyncness};

pub(crate) fn generate(input: &ItemStory, asyncness: Asyncness) -> TokenStream {
    quote! {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[ignore]
    fn test_generate_sync() {
        let input = syn::parse_quote! {
            trait UserStory {
                #[step("step1")]
                fn step1();
                #[step("step2: {name}", name = "ryo")]
                fn step2(name: &str);
            }
        };
        let actual = generate(&input, Asyncness::Sync);
        let expected = quote! {
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct StepId {
                index: usize,
                _phantom: std::marker::PhantomData<T>,
            }
            impl <T> narrative::step::StepId<T> for StepId {
                fn index(&self) -> usize {
                    self.index
                }
            }
            #[derive(Clone, Copy)]
            pub struct Step<T, E> {
                index: usize,
                step_fn: fn(&mut T) -> Result<(), E>,
                step_text: fn() -> String,
                args: Args,
            }
            impl <T, E> narrative::step::Step for Step<T, E> {
                type Story = T;
                type Error = E;
                type StepId: StepId<Self::Story>;
                fn run(&self, story: &mut Self::Story) -> Result<(), Self::Error> {
                    (self.step_fn)(story)
                }
                fn step_text(&self) -> String {
                    (self.step_text)()
                }
                fn args(&self) -> Args {
                    self.args
                }
                fn id(&self) -> Self::StepId {
                    StepId {
                        index: self.index,
                        _phantom: std::marker::PhantomData,
                    }
                }
            }
            #[derive(Clone, Copy)]
            pub struct Steps<T> {
                steps: &'static [T],
            }
            impl <T> narrative::step::Steps for Steps<T> {
                type Step = T;
                fn get(&self, id: <Self::Step as Step>::StepId) -> &'static Self::Step;
                fn iter(&self) -> std::slice::Iter<'static, Self::Step>;
            }
            impl <T> IntoIterator for Steps<T> {
                type Item = &'static T;
                type IntoIter = std::slice::Iter<'static, T>;
                fn into_iter(self) -> Self::IntoIter {
                    self.steps.iter()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
