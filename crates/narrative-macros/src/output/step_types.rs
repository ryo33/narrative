use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

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
                 ident, mod_ident, ..
             }| quote!(Self::#ident => steps::#mod_ident::step_text(),),
        )
        .collect();
    let step_idents: MatchArms = steps
        .iter()
        .map(
            |StepSegments {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => steps::#mod_ident::__STEP_ID,),
        )
        .collect();
    let step_args = steps
        .iter()
        .map(
            |StepSegments {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => steps::#mod_ident::__ARGS.iter().copied(),),
        )
        .collect::<MatchArms>()
        .cast_as(quote!(std::iter::Empty<StepArg>));
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
    let step_stories = steps
        .iter()
        .map(
            |StepSegments {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => steps::#mod_ident::dyn_nested_story(),),
        )
        .collect::<MatchArms>()
        .cast_as(quote!(Option<narrative::story::DynStoryContext>));
    let to_dyn_arms = steps
        .iter()
        .map(
            |StepSegments {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => steps::#mod_ident::DYN_STEP,),
        )
        .collect::<MatchArms>();
    let steps_def = steps
        .iter()
        .map(|StepSegments { step_def, .. }| step_def)
        .collect::<Vec<_>>();

    quote! {
        mod steps {
            use super::*;
            #(#steps_def)*
        }

        #[derive(Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum Step {
            #(#step_names),*
        }
        impl narrative::step::Step for Step {
            #[inline]
            fn step_text(&self) -> String {
                #step_texts
            }
            #[inline]
            fn step_id(&self) -> &'static str {
                #step_idents
            }
            #[inline]
            fn args(&self) -> impl Iterator<Item = impl narrative::step::StepArg + 'static> + 'static {
                #step_args
            }
            #[inline]
            fn story(&self) -> impl narrative::story::StoryContext<Step = Self> + 'static {
                StoryContext::default()
            }
            #[inline]
            fn nested_story(&self) -> Option<impl narrative::story::StoryContext + Send + 'static> {
                #step_stories
            }
        }
        impl Step {
            pub fn to_dyn(&self) -> narrative::step::DynStep {
                #to_dyn_arms
            }
        }

        impl <T: #story_ident> narrative::step::Run<T, T::Error> for Step {
            #[inline]
            fn run(&self, story: &mut T) -> Result<(), T::Error> {
                use narrative::runner::StoryRunner as _;
                let mut runner = narrative::runner::DefaultStoryRunner;
                self.run_with_runner(story, &mut runner)
            }
            #[inline]
            fn run_with_runner(&self, story: &mut T, runner: &mut impl narrative::runner::StoryRunner<T::Error>) -> Result<(), T::Error> {
                use narrative::runner::StoryRunner as _;
                #step_runs
            }
        }

        impl <T: #async_story_ident> narrative::step::RunAsync<T, T::Error> for Step {
            #[inline]
            async fn run_async(&self, story: &mut T) -> Result<(), T::Error> {
                use narrative::runner::AsyncStoryRunner as _;
                let mut runner = narrative::runner::DefaultStoryRunner;
                self.run_with_runner_async(story, &mut runner).await
            }
            #[inline]
            async fn run_with_runner_async(&self, story: &mut T, runner: &mut impl narrative::runner::AsyncStoryRunner<T::Error>) -> Result<(), T::Error> {
                use narrative::runner::AsyncStoryRunner as _;
                #step_runs_async
            }
        }
    }
}

pub struct StepSegments<'a> {
    ident: &'a syn::Ident,
    mod_ident: syn::Ident,
    run: TokenStream,
    run_async: TokenStream,
    step_def: StepDef,
}

pub struct StepDef {
    mod_ident: syn::Ident,
    step_text: TokenStream,
    step_id: TokenStream,
    args: TokenStream,
    story: TokenStream,
    nested_story: TokenStream,
    dyn_step: TokenStream,
}

impl ToTokens for StepDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StepDef {
            mod_ident,
            step_text,
            step_id,
            args,
            story,
            nested_story,
            dyn_step,
        } = &self;
        tokens.extend(quote! {
            pub mod #mod_ident {
                use super::*;
                #step_text
                #step_id
                #args
                #story
                #nested_story
                #dyn_step
            }
        });
    }
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
    let const_bindings = step.generate_const_bindings(story);
    let extracted_format_args = step.extract_format_args();

    // Collect format args from step attributes first
    let attr_names = step
        .attr
        .args
        .iter()
        .filter(|arg| extracted_format_args.contains(&arg.ident.to_string()))
        .map(|arg| arg.ident.to_string())
        .collect::<BTreeSet<_>>();

    let format_args_from_attr = step.attr.args.iter().filter_map(|arg| {
        if extracted_format_args.contains(&arg.ident.to_string()) {
            let value = &arg.value;
            Some((&arg.ident, quote!(#value)))
        } else {
            None
        }
    });
    let format_args_from_global = story.consts().filter_map(|StoryConst { raw, default }| {
        if extracted_format_args.contains(&raw.ident.to_string())
            && !attr_names.contains(&raw.ident.to_string())
        {
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

    let nested_story = if let Some(sub_story_path) = step.sub_story_path() {
        let context_path = &sub_story_path.context_path();
        quote! {
            pub fn nested_story() -> #context_path {
                #context_path::default()
            }
            pub fn dyn_nested_story() -> Option<narrative::story::DynStoryContext> {
                Some(nested_story().to_dyn())
            }
        }
    } else {
        quote! {
            pub fn dyn_nested_story() -> Option<narrative::story::DynStoryContext> {
                None
            }
        }
    };
    let (run, run_async) = if step.has_sub_story() {
        (
            quote! {
                #(#const_bindings)*
                #(#step_args_assignments)*
                let mut sub_story = T::#step_name(story #(,#args)*)?;
                let story = sub_story.get_context();
                runner.run_nested_story(Step::#step_name, story, &mut sub_story)?;
                Ok(())
            },
            quote! {
                #(#const_bindings)*
                #(#step_args_assignments)*
                let mut sub_story = T::#step_name(story #(,#args)*)?;
                let story = sub_story.get_context();
                runner.run_nested_story_async(Step::#step_name, story, &mut sub_story).await?;
                Ok(())
            },
        )
    } else {
        (
            quote! {
                #(#const_bindings)*
                #(#step_args_assignments)*
                T::#step_name(story #(,#args)*)
            },
            quote! {
                #(#const_bindings)*
                #(#step_args_assignments)*
                T::#step_name(story #(,#args)*).await
            },
        )
    };

    let args_len = args.len();
    let dyn_args = if args.is_empty() {
        quote!(Box::new(std::iter::empty()))
    } else {
        quote!(Box::new(__ARGS.iter().map(|arg| arg.to_dyn())))
    };

    let ident = &step.inner.sig.ident;
    let mod_ident = format_ident!("mod_{}", ident);

    let step_def = StepDef {
        mod_ident: mod_ident.clone(),
        step_text: quote! {
            pub fn step_text() -> String {
                format!(#step_text #(#format_args)*)
            }
        },
        step_id: quote!(
            pub const __STEP_ID: &str = stringify!(#step_name);
        ),
        args: quote!(pub const __ARGS: [StepArg; #args_len] = [#(StepArg(StepArgInner::#step_name(args::#step_name::#args))),*];),
        story: quote!(
            pub const __STORY: StoryContext = StoryContext;
        ),
        nested_story,
        dyn_step: quote! {
            pub const DYN_STEP: narrative::step::DynStep = narrative::step::DynStep::new(
                step_text,
                __STEP_ID,
                || #dyn_args,
                || __STORY.to_dyn(),
                dyn_nested_story,
            );
        },
    };

    StepSegments {
        ident,
        mod_ident,
        run,
        run_async,
        step_def,
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1")
                }
            }
            .to_string()
        );
        assert_eq!(
            actual.step_def.step_id.to_string(),
            quote! {
                pub const __STEP_ID: &str = stringify!(my_step1);
            }
            .to_string()
        );
        assert_eq!(
            actual.step_def.args.to_string(),
            quote! {
                pub const __ARGS: [StepArg; 0usize] = [];
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name}", name = "ryo")
                }
            }
            .to_string()
        );
        assert_eq!(
            actual.step_def.args.to_string(),
            quote! {
                pub const __ARGS: [StepArg; 1usize] = [StepArg(StepArgInner::my_step1(args::my_step1::name))];
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name}", name = "ryo")
                }
            }
            .to_string()
        );
        assert_eq!(
            actual.step_def.args.to_string(),
            quote! {
                pub const __ARGS: [StepArg; 1usize] = [StepArg(StepArgInner::my_step1(args::my_step1::name))];
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name}", name = "ryo")
                }
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name} {age}", name = "ryo")
                }
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
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name:?}", name = "ryo")
                }
            }
            .to_string()
        );
    }

    #[test]
    fn test_sub_story_step() {
        let step = parse_quote! {
            #[step(story: SubStory, "run sub story")]
            fn run_sub();
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
                let mut sub_story = T::run_sub(story)?;
                let story = sub_story.get_context();
                runner.run_nested_story(Step::run_sub, story, &mut sub_story)?;
                Ok(())
            }
            .to_string()
        );

        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                let mut sub_story = T::run_sub(story)?;
                let story = sub_story.get_context();
                runner.run_nested_story_async(Step::run_sub, story, &mut sub_story).await?;
                Ok(())
            }
            .to_string()
        );

        assert_eq!(
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("run sub story")
                }
            }
            .to_string()
        );

        // Test nested_story field for sub-story
        assert_eq!(
            actual.step_def.nested_story.to_string(),
            quote! {
                pub fn nested_story() -> SubStoryContext {
                    SubStoryContext::default()
                }
                pub fn dyn_nested_story() -> Option<narrative::story::DynStoryContext> {
                    Some(nested_story().to_dyn())
                }
            }
            .to_string()
        );
    }

    #[test]
    fn test_sub_story_step_with_args() {
        let step = parse_quote! {
            #[step(story: SubStory, "run sub story with {param}", param = 42)]
            fn run_sub(param: i32);
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
                let param: i32 = 42;
                let mut sub_story = T::run_sub(story, param)?;
                let story = sub_story.get_context();
                runner.run_nested_story(Step::run_sub, story, &mut sub_story)?;
                Ok(())
            }
            .to_string()
        );

        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                let param: i32 = 42;
                let mut sub_story = T::run_sub(story, param)?;
                let story = sub_story.get_context();
                runner.run_nested_story_async(Step::run_sub, story, &mut sub_story).await?;
                Ok(())
            }
            .to_string()
        );

        assert_eq!(
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("run sub story with {param}", param = 42)
                }
            }
            .to_string()
        );

        assert_eq!(
            actual.step_def.args.to_string(),
            quote! {
                pub const __ARGS: [StepArg; 1usize] = [StepArg(StepArgInner::run_sub(args::run_sub::param))];
            }
            .to_string()
        );
    }

    #[test]
    fn test_const_binding_in_arg_assignment() {
        // Step uses a story constant (MY_CONST) inside an attribute argument expression.
        let step = parse_quote! {
            #[step("Step 1: {param}", param = MY_CONST * 2)]
            fn my_step(param: i32);
        };
        let story_syntax = parse_quote! {
            trait MyStory {
                const MY_CONST: i32 = 10;
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);

        // run (sync)
        assert_eq!(
            actual.run.to_string(),
            quote! {
                let MY_CONST: i32 = 10;
                let param: i32 = MY_CONST * 2;
                T::my_step(story, param)
            }
            .to_string()
        );

        // run_async
        assert_eq!(
            actual.run_async.to_string(),
            quote! {
                let MY_CONST: i32 = 10;
                let param: i32 = MY_CONST * 2;
                T::my_step(story, param).await
            }
            .to_string()
        );
    }

    #[test]
    fn test_step_text_override_global_assignment() {
        let step = parse_quote! {
            #[step("Step 1: {name}", name = "override")]
            fn my_step1(name: &str);
        };
        let story_syntax = parse_quote! {
            trait UserStory {
                const name: &str = "original";
                #step
            }
        };
        let actual = generate_step(&story_syntax, &step);

        // Step attribute value should override global constant in format args
        assert_eq!(
            actual.step_def.step_text.to_string(),
            quote! {
                pub fn step_text() -> String {
                    format!("Step 1: {name}", name = "override")
                }
            }
            .to_string()
        );

        // The run method binds the value from the attribute
        assert_eq!(
            actual.run.to_string(),
            quote! {
                let name: &str = "override";
                T::my_step1(story, name)
            }
            .to_string()
        );
    }
}
