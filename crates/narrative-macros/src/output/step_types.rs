use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    item_story::{ItemStory, StoryItem, StoryStep},
    Asyncness,
};

pub(crate) fn generate(story: &ItemStory, asyncness: Asyncness) -> TokenStream {
    let step_names = story.steps().map(|step| step.inner.sig.ident);
    let steps: Vec<_> = story
        .items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| match item {
            StoryItem::Step(step) => Some(generate_step(story, idx, step, asyncness)),
            _ => None,
        })
        .collect();
    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct StepId {
            index: usize,
        }
        impl StepId {
            pub fn new(index: usize) -> Self {
                Self {
                    index,
                }
            }
        }
        impl narrative::step::StepId for StepId {
            fn index(&self) -> usize {
                self.index
            }
        }
        #[derive(Clone, Copy, Default, narrative::Serialize)]
        #[serde(transparent)]
        pub struct Step<T>(T);

        impl <T: std::fmt::Debug> std::fmt::Debug for Step<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        mod steps {
            use super::*;
            #[allow(non_camel_case_types)]
            pub enum Step {
                #(#step_names),*
            }
            #(#steps)*
        }
    }
}

pub struct StepSegments {
    run: TokenStream,
    step_text: TokenStream,
    args: TokenStream,
    id: TokenStream,
}

fn generate_step(
    story: &ItemStory,
    idx: usize,
    step: &StoryStep,
    asyncness: Asyncness,
) -> TokenStream {
    let step_name = &step.inner.sig.ident;
    let step_text = &step.attr.text;
    // We don't filter out unused step args here to generate unused warnings.
    let step_args_assignments = step.attr.args.iter().map(|arg| {
        let name = &arg.ident;
        let value = &arg.value;
        quote! {
            let #name = #value;
        }
    });
    let format_args_from_attr = step.attr.args.iter().filter_map(|arg| {
        if step_text
            .value()
            .contains(&("{".to_string() + &arg.ident.to_string() + "}"))
        {
            let value = &arg.value;
            Some((&arg.ident, quote!(#value)))
        } else {
            None
        }
    });
    let format_args_from_global = story.items.iter().filter_map(|item| match item {
        StoryItem::Let(assignment) => {
            let syn::Pat::Ident(pat_ident) = assignment.pat.as_ref() else {
                return None;
            };
            if step_text
                .value()
                .contains(&("{".to_string() + &pat_ident.ident.to_string() + "}"))
            {
                let expr = &assignment.expr;
                Some((&pat_ident.ident, quote!(#expr)))
            } else {
                None
            }
        }
        _ => None,
    });
    let format_args = format_args_from_attr
        .chain(format_args_from_global)
        .map(|(ident, expr)| {
            quote! {
                , #ident = #expr
            }
        });
    let used_global_assignments = story.items.iter().filter_map(|item| match item {
        StoryItem::Let(assignment) => {
            if step.inner.sig.inputs.iter().any(|arg| {
                let syn::FnArg::Typed(syn::PatType { pat, .. }) = arg else {
                    return false;
                };
                pat == &assignment.pat
            }) {
                Some(assignment)
            } else {
                None
            }
        }
        _ => None,
    });
    let args = step.inner.sig.inputs.iter().filter_map(|arg| match arg {
        syn::FnArg::Typed(syn::PatType { pat, .. }) => Some(pat),
        _ => None,
    });
    let story_ident = &story.ident;
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct #step_name<T, E> {
            phantom: std::marker::PhantomData<(T, E)>,
        }

        impl <T: #story_ident> Step<#step_name<T, T::Error>> {
            fn run(&self, story: &mut T) -> Result<(), T::Error> {
                #(#used_global_assignments)*
                #(#step_args_assignments)*
                T::#step_name(story #(,#args)*)
            }
            fn step_text(&self) -> String {
                format!(#step_text #(#format_args)*)
            }
            fn args(&self) -> Self::ArgIter {
                Default::default()
            }
            fn id(&self) -> Self::Id {
                StepId::new(#idx)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse2;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple() {
        let step = quote! {
            #[step("Step 1")]
            fn my_step1();
        };
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, 42, &parse2(step).unwrap(), Asyncness::Sync);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug, Default)]
            pub struct my_step1<T, E> {
                phantom: std::marker::PhantomData<(T, E)>,
            }

            impl <T: UserStory> narrative::step::Step<T, T::Error> for Step<my_step1<T, T::Error>> {
                type Id = StepId;
                type Arg = StepArg<args::my_step1>;
                type ArgIter = std::slice::Iter<'static, Self::Arg>;
                fn run(&self, story: &mut T) -> Result<(), T::Error> {
                    T::my_step1(story)
                }
                fn step_text(&self) -> String {
                    format!("Step 1")
                }
                fn args(&self) -> Self::ArgIter {
                    Default::default()
                }
                fn id(&self) -> Self::Id {
                    StepId::new(42usize)
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
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, 1, &parse2(step).unwrap(), Asyncness::Sync);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug, Default)]
            pub struct my_step1<T, E> {
                phantom: std::marker::PhantomData<(T, E)>,
            }

            impl <T: UserStory> narrative::step::Step<T, T::Error> for Step<my_step1<T, T::Error>> {
                type Id = StepId;
                type Arg = StepArg<args::my_step1>;
                type ArgIter = std::slice::Iter<'static, Self::Arg>;

                fn run(&self, story: &mut T) -> Result<(), T::Error> {
                    let name = "ryo";
                    T::my_step1(story, name)
                }
                fn step_text(&self) -> String {
                    format!("Step 1: {name}", name = "ryo")
                }
                fn args(&self) -> Self::ArgIter {
                    Default::default()
                }
                fn id(&self) -> Self::Id {
                    StepId::new(1usize)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    /// User can get unused warnings for step attr args.
    fn test_unused_step_attr_args() {
        let step = quote! {
            #[step("Step 1: {name}", name = "ryo")]
            fn my_step1();
        };
        let story_syntax = syn::parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, 1, &parse2(step).unwrap(), Asyncness::Sync);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, Debug, Default)]
            pub struct my_step1<T, E> {
                phantom: std::marker::PhantomData<(T, E)>,
            }

            impl <T: UserStory> narrative::step::Step<T, T::Error> for Step<my_step1<T, T::Error>> {
                type Id = StepId;
                type Arg = StepArg<args::my_step1>;
                type ArgIter = std::slice::Iter<'static, Self::Arg>;

                fn run(&self, story: &mut T) -> Result<(), T::Error> {
                    let name = "ryo";
                    T::my_step1(story)
                }
                fn step_text(&self) -> String {
                    format!("Step 1: {name}", name = "ryo")
                }
                fn args(&self) -> Self::ArgIter {
                    Default::default()
                }
                fn id(&self) -> Self::Id {
                    StepId::new(1usize)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_global_assignments() {}

    #[test]
    fn test_format_arg_insufficient() {}
}
