use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    item_story::{story_const::StoryConst, ItemStory, StoryStep},
    output::MatchArms,
};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let story_ident = &story.ident;
    let async_story_ident = format_ident!("Async{}", story_ident);
    let step_names = story.steps().map(|step| &step.inner.sig.ident);
    let steps: Vec<_> = story
        .steps()
        .map(|step| generate_step(story, step))
        .collect();
    let step_texts: MatchArms = steps
        .iter()
        .map(
            |StepSegments {
                 ident, step_text, ..
             }| quote!(Self::#ident => { #step_text }),
        )
        .collect();
    let step_idents: MatchArms = steps
        .iter()
        .map(|StepSegments { ident, idents, .. }| quote!(Self::#ident => { #idents }))
        .collect();
    let step_args: MatchArms = steps
        .iter()
        .map(|StepSegments { ident, args, .. }| quote!(Self::#ident => { #args }))
        .collect();
    let step_runs: MatchArms = steps
        .iter()
        .map(|StepSegments { ident, run, .. }| quote!(Self::#ident => { #run }))
        .collect();
    let step_runs_async: MatchArms = steps
        .iter()
        .map(
            |StepSegments {
                 ident, run_async, ..
             }| quote!(Self::#ident => { #run_async }),
        )
        .collect();

    quote! {
        #[allow(non_camel_case_types)]
        pub enum Step {
            #(#step_names),*
        }
        impl narrative::step::Step for Step {
            type Arg = StepArg;
            type ArgIter = std::slice::Iter<'static, Self::Arg>;
            #[inline]
            fn step_text(&self) -> String {
                #step_texts
            }
            #[inline]
            fn step_id(&self) -> &'static str {
                #step_idents
            }
            #[inline]
            fn args(&self) -> Self::ArgIter {
                #step_args
            }
        }

        impl <T: #story_ident> narrative::step::Run<T, T::Error> for Step {
            #[inline]
            fn run(&self, story: &mut T) -> Result<(), T::Error> {
                #step_runs
            }
        }

        impl <T: #async_story_ident> narrative::step::RunAsync<T, T::Error> for Step {
            #[inline]
            async fn run_async(&self, story: &mut T) -> Result<(), T::Error> {
                #step_runs_async
            }
        }
    }
}

pub struct StepSegments<'a> {
    ident: &'a syn::Ident,
    run: TokenStream,
    run_async: TokenStream,
    step_text: TokenStream,
    args: TokenStream,
    idents: TokenStream,
}

fn generate_step<'a>(story: &'a ItemStory, step: &'a StoryStep) -> StepSegments<'a> {
    let step_name = &step.inner.sig.ident;
    let step_text = &step.attr.text;
    // We don't filter out unused step args here to generate unused warnings.
    let step_args_assignments: Vec<_> = step
        .attr
        .args
        .iter()
        .map(|arg| {
            let name = &arg.ident;
            let ty = step
                .fn_args()
                .find(|(ident, _)| *ident == name)
                .map(|(_, ty)| quote!(:#ty));
            let value = &arg.value;
            quote! {
                let #name #ty = #value;
            }
        })
        .collect();
    let extracted_format_args = step.extract_format_args();
    let format_args_from_attr = step.attr.args.iter().filter_map(|arg| {
        if extracted_format_args.contains(&arg.ident.to_string()) {
            let value = &arg.value;
            Some((&arg.ident, quote!(#value)))
        } else {
            None
        }
    });
    let format_args_from_global = story.consts().filter_map(|StoryConst { raw, default }| {
        if extracted_format_args.contains(&raw.ident.to_string()) {
            let expr = &default.1;
            Some((&raw.ident, quote!(#expr)))
        } else {
            None
        }
    });
    let format_args = format_args_from_attr
        .chain(format_args_from_global)
        .map(|(ident, expr)| {
            quote! {
                , #ident = #expr
            }
        });
    let args: Vec<_> = step.fn_args().map(|(ident, _)| ident).collect();

    StepSegments {
        ident: &step.inner.sig.ident,
        run: quote! {
            #(#step_args_assignments)*
            T::#step_name(story #(,#args)*)
        },
        run_async: quote! {
            #(#step_args_assignments)*
            T::#step_name(story #(,#args)*).await
        },
        step_text: quote! {
            format!(#step_text #(#format_args)*)
        },
        idents: quote! {
            stringify!(#step_name)
        },
        args: quote! {
            [#(StepArg(StepArgInner::#step_name(args::#step_name::#args))),*].iter()
        },
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple() {
        let step = parse_quote! {
            #[step("Step 1")]
            fn my_step1();
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.run.to_string(),
            quote! {
                T::my_step1(story)
            }
            .to_string()
        );
        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                T::my_step1(story).await
            }
            .to_string()
        );
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1")
            }
            .to_string()
        );
        assert_eq!(
            actual.idents.to_string(),
            quote! {
                stringify!(my_step1)
            }
            .to_string()
        );
        assert_eq!(
            actual.args.to_string(),
            quote! {
                [].iter()
            }
            .to_string()
        );
    }

    #[test]
    fn use_step_attr_args() {
        let step = parse_quote! {
            #[step("Step 1: {name}", name = "ryo")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.run.to_string(),
            quote! {
                let name: &str = "ryo";
                T::my_step1(story, name)
            }
            .to_string()
        );
        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                let name: &str = "ryo";
                T::my_step1(story, name).await
            }
            .to_string()
        );
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1: {name}", name = "ryo")
            }
            .to_string()
        );
        assert_eq!(
            actual.args.to_string(),
            quote! {
                [StepArg(StepArgInner::my_step1(args::my_step1::name))].iter()
            }
            .to_string()
        );
    }

    #[test]
    /// User can get unused warnings for step attr args.
    fn test_unused_step_attr_args() {
        let step = parse_quote! {
            #[step("Step 1: {name}", name = "ryo", unused = "unused")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.run.to_string(),
            quote! {
                let name: &str = "ryo";
                let unused = "unused";
                T::my_step1(story, name)
            }
            .to_string()
        );
        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                let name: &str = "ryo";
                let unused = "unused";
                T::my_step1(story, name).await
            }
            .to_string()
        );
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1: {name}", name = "ryo")
            }
            .to_string()
        );
        assert_eq!(
            actual.args.to_string(),
            quote! {
                [StepArg(StepArgInner::my_step1(args::my_step1::name))].iter()
            }
            .to_string()
        );
    }

    #[test]
    /// Outputs don't include global assignments, and uses the one defined with match statement.
    /// But step_text refers global assignments.
    fn test_run_does_not_include_global_assignments() {
        let step = parse_quote! {
            #[step("Step 1: {name}")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                const name: &str = "ryo";
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.run.to_string(),
            quote! {
                T::my_step1(story, name)
            }
            .to_string()
        );
        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                T::my_step1(story, name).await
            }
            .to_string()
        );
    }

    #[test]
    fn test_step_text_can_refer_global_assignment() {
        let step = parse_quote! {
            #[step("Step 1: {name}")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                const name: &str = "ryo";
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1: {name}", name = "ryo")
            }
            .to_string()
        );
    }

    #[test]
    /// User can get insufficient format args error.
    fn test_format_arg_insufficient() {
        let step = parse_quote! {
            #[step("Step 1: {name} {age}", name = "ryo")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1: {name} {age}", name = "ryo")
            }
            .to_string()
        );
    }

    #[test]
    fn test_format_with_debug() {
        let step = parse_quote! {
            #[step("Step 1: {name:?}", name = "ryo")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);
        assert_eq!(
            actual.step_text.to_string(),
            quote! {
                format!("Step 1: {name:?}", name = "ryo")
            }
            .to_string()
        );
    }
}
