// important! We don't make step args types camel case, to support rich rename experience.
// To avoid name conflict, we use dedicated module for args.
// enum dispatched by step name and step arg name.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};

use crate::{
    item_story::{ItemStory, StoryStep},
    make_static,
    output::MatchArms,
    pretty_print_expr, pretty_print_type,
};

pub(crate) fn generate(story: &ItemStory) -> TokenStream {
    let steps_with_args = story
        .steps()
        .filter(|step| step.fn_args().next().is_some())
        .collect::<Vec<_>>();
    if steps_with_args.is_empty() {
        return quote! {
            /// This is placeholder and never be constructed.
            pub type StepArg = narrative::step::DynStepArg;
        };
    }
    let step_names: Vec<_> = steps_with_args
        .iter()
        .map(|step| &step.inner.sig.ident)
        .collect();
    let step_enums = steps_with_args.iter().cloned().map(generate_step_enum);
    let arg_impls = steps_with_args
        .iter()
        .map(|step| generate_arg_impl(story, step));
    let serde_impls = steps_with_args.iter().cloned().map(generate_serialize_impl);
    let debug_impls = steps_with_args.iter().cloned().map(generate_debug_impl);
    let arg_values = steps_with_args.iter().cloned().map(generate_arg_values);
    let arg_value_debug_arms = steps_with_args
        .iter()
        .map(|step| {
            let step_ident = &step.inner.sig.ident;
            quote!(Self::#step_ident(arg) => arg.fmt(f),)
        })
        .collect::<MatchArms>();
    let step_arg_value_arms = steps_with_args
        .iter()
        .map(|step| {
            let step_ident = &step.inner.sig.ident;
            quote!(StepArgInner::#step_ident(arg) => arg.value(),)
        })
        .collect::<MatchArms>()
        .match_target(quote!(self.0))
        .cast_as(quote!(narrative::value::BoxedValue));

    quote! {
        #[derive(Clone, Copy, narrative::serde::Serialize)]
        #[serde(transparent, crate = "narrative::serde")]
        pub struct StepArg(StepArgInner);

        #[derive(Clone, Copy, narrative::serde::Serialize)]
        #[allow(non_camel_case_types)]
        #[serde(untagged, crate = "narrative::serde")]
        enum StepArgInner {
            #(#step_names(args::#step_names)),*
        }

        #[allow(non_camel_case_types)]
        #[derive(narrative::serde::Serialize)]
        #[serde(untagged, crate = "narrative::serde")]
        enum ArgValue {
            #(#step_names(arg_values::#step_names)),*
        }

        impl std::fmt::Debug for ArgValue {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #arg_value_debug_arms
            }
        }

        impl narrative::step::StepArg for StepArg {
            #[inline]
            fn name(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.name()),*
                }
            }
            #[inline]
            fn ty(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.ty()),*
                }
            }
            #[inline]
            fn expr(&self) -> &'static str {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.expr()),*
                }
            }
            #[inline]
            fn value(&self) -> impl narrative::value::Value {
                #step_arg_value_arms
            }
        }

        impl StepArg {
            pub fn to_dyn(&self) -> narrative::step::DynStepArg {
                match self.0 {
                    #(StepArgInner::#step_names(arg) => arg.to_dyn()),*
                }
            }
        }

        impl std::fmt::Debug for StepArg {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                use narrative::step::StepArg;
                write!(f, "{name}: {ty} = {expr}", name = self.name(), ty = self.ty(), expr = self.expr())
            }
        }

        mod arg_values {
            use super::*;
            #(#arg_values)*
        }
        mod args {
            use super::*;

            #(#step_enums)*
            #(#arg_impls)*
            #(#serde_impls)*
            #(#debug_impls)*
        }
    }
}

fn generate_debug_impl(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    quote! {
        impl std::fmt::Debug for #step_ident {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{name}: {ty} = {expr}", name = self.name(), ty = self.ty(), expr = self.expr())
            }
        }
    }
}

fn generate_step_enum(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    let variants = step.fn_args().map(|(ident, _)| ident);
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        pub(super) enum #step_ident {
            #(#variants,)*
        }
    }
}

fn generate_serialize_impl(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    quote! {
        impl narrative::serde::Serialize for #step_ident {
            #[inline]
            fn serialize<T: narrative::serde::Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
                use narrative::serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("name", self.name())?;
                map.serialize_entry("ty", self.ty())?;
                map.serialize_entry("expr", self.expr())?;
                map.serialize_entry("value", &self.value())?;
                map.end()
            }
        }
    }
}

fn generate_arg_values(step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    let variants = step.fn_args().map(|(ident, ty)| {
        let static_ty = make_static(ty);
        quote! {
            #ident(#static_ty)
        }
    });
    let step_arg_debug_arms = step
        .fn_args()
        .map(|(ident, _)| quote!(Self::#ident(arg) => arg.fmt(f),))
        .collect::<MatchArms>();
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(narrative::serde::Serialize)]
        #[serde(untagged, crate = "narrative::serde")]
        pub(super) enum #step_ident {
            #(#variants,)*
        }

        impl std::fmt::Debug for #step_ident {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #step_arg_debug_arms
            }
        }
    }
}

fn generate_arg_impl(story: &ItemStory, step: &StoryStep) -> TokenStream {
    let step_ident = &step.inner.sig.ident;
    // Create identifier for the outer step args module (e.g., my_step_args)
    let step_ident_args_mod = format_ident!("{}_args", step_ident);

    struct FnArg<'a> {
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
        mod_ident: syn::Ident,
    }

    let args = step
        .fn_args()
        .map(|(ident, ty)| FnArg {
            ident,
            ty,
            mod_ident: format_ident!("mod_{}", ident),
        })
        .collect::<Vec<_>>();

    let name_arms = args
        .iter()
        .map(
            |FnArg {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => #step_ident_args_mod::#mod_ident::__NAME,),
        )
        .collect::<MatchArms>();
    let ty_arms = args
        .iter()
        .map(
            |FnArg {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => #step_ident_args_mod::#mod_ident::__TY,),
        )
        .collect::<MatchArms>();
    let expr_arms = args
        .iter()
        .map(
            |FnArg {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => #step_ident_args_mod::#mod_ident::__EXPR,),
        )
        .collect::<MatchArms>();
    let value_arms = args
        .iter()
        .map(|FnArg { ident, mod_ident, .. }| quote!(Self::#ident => ArgValue::#step_ident(arg_values::#step_ident::#ident(#step_ident_args_mod::#mod_ident::value())),))
        .collect::<MatchArms>();
    let to_dyn_arms = args
        .iter()
        .map(
            |FnArg {
                 ident, mod_ident, ..
             }| quote!(Self::#ident => #step_ident_args_mod::#mod_ident::DYN_STEP_ARG,),
        )
        .collect::<MatchArms>();
    let result = args.iter().map(|FnArg { ident, ty, mod_ident }| {
        let Some(expr) = step
            .find_attr_arg(ident)
            .or_else(|| story.find_assignments(ident)) else {
                return Err(quote_spanned! { ident.span() => compile_error!("No attr arg or assignment found") });
            };

        let static_ty = make_static(ty);

        let ty_str = pretty_print_type(ty);
        let expr_str = pretty_print_expr(expr);

        Ok(quote! {
            pub mod #mod_ident {
                use super::*;
                pub const __NAME: &str = stringify!(#ident);
                pub const __TY: &str = #ty_str;
                pub const __EXPR: &str = #expr_str;
                #[inline]
                pub fn value() -> #static_ty {
                    #expr
                }
                pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                    __NAME,
                    __TY,
                    __EXPR,
                    || narrative::value::BoxedValue::new(value()),
                    || narrative::value::BoxedValue::new(#step_ident::#ident)
                );
            }
        })
    })
        .collect::<Result<Vec<_>, _>>();
    let defs = match result {
        Ok(arms) => arms,
        Err(err) => {
            return err;
        }
    };

    quote! {
        pub mod #step_ident_args_mod {
            use super::*;
            #(#defs)*
        }
        impl #step_ident {
            #[inline]
            pub(super) fn name(&self) -> &'static str {
                #name_arms
            }
            #[inline]
            pub(super) fn ty(&self) -> &'static str {
                #ty_arms
            }
            #[inline]
            pub(super) fn expr(&self) -> &'static str {
                #expr_arms
            }
            #[inline]
            pub(super) fn value(&self) -> ArgValue {
                #value_arms
            }
            #[inline]
            pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                #to_dyn_arms
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use syn::parse_quote;

    #[test]
    fn simple() {
        let step: StoryStep = parse_quote! {
            #[step("Step 1")]
            fn my_step1();
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod my_step1_args {
                use super::*;
            }
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    unreachable!()
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    unreachable!()
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    unreachable!()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn use_step_attr_args() {
        let step: StoryStep = parse_quote! {
            #[step("Step 1: {name}", name = "ryo")]
            fn my_step1(name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod my_step1_args {
                use super::*;
                pub mod mod_name {
                    use super::*;
                    pub const __NAME: &str = stringify!(name);
                    pub const __TY: &str = "&str";
                    pub const __EXPR: &str = "\"ryo\"";
                    #[inline]
                    pub fn value() -> &'static str {
                        "ryo"
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(my_step1::name)
                    );
                }
            }
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::name => my_step1_args::mod_name::__NAME,
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::name => my_step1_args::mod_name::__TY,
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::name => my_step1_args::mod_name::__EXPR,
                    }
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    match self {
                        Self::name => ArgValue::my_step1(arg_values::my_step1::name(my_step1_args::mod_name::value())),
                    }
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    match self {
                        Self::name => my_step1_args::mod_name::DYN_STEP_ARG,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    /// Find global assignments
    fn test_global_assignments() {
        let step: StoryStep = parse_quote! {
            #[step("Step 1")]
            fn my_step1(id: UserId, name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                const id: UserId = UserId::new();
                const name: &str = "Alice";
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod my_step1_args {
                use super::*;
                pub mod mod_id {
                    use super::*;
                    pub const __NAME: &str = stringify!(id);
                    pub const __TY: &str = "UserId";
                    pub const __EXPR: &str = "UserId::new()";
                    #[inline]
                    pub fn value() -> UserId {
                        UserId::new()
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(my_step1::id)
                    );
                }
                pub mod mod_name {
                    use super::*;
                    pub const __NAME: &str = stringify!(name);
                    pub const __TY: &str = "&str";
                    pub const __EXPR: &str = "\"Alice\"";
                    #[inline]
                    pub fn value() -> &'static str {
                        "Alice"
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(my_step1::name)
                    );
                }
            }
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__NAME,
                        Self::name => my_step1_args::mod_name::__NAME,
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__TY,
                        Self::name => my_step1_args::mod_name::__TY,
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__EXPR,
                        Self::name => my_step1_args::mod_name::__EXPR,
                    }
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    match self {
                        Self::id => ArgValue::my_step1(arg_values::my_step1::id(my_step1_args::mod_id::value())),
                        Self::name => ArgValue::my_step1(arg_values::my_step1::name(my_step1_args::mod_name::value())),
                    }
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    match self {
                        Self::id => my_step1_args::mod_id::DYN_STEP_ARG,
                        Self::name => my_step1_args::mod_name::DYN_STEP_ARG,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    /// Find local assignments first
    fn test_local_assignments() {
        let step: StoryStep = parse_quote! {
            #[step("Step 1", name = "Bob")]
            fn my_step1(id: UserId, name: &str);
        };
        let story_syntax = syn::parse_quote! {
            trait User {
                const id: UserId = UserId::new();
                const name: &str = "Alice";
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod my_step1_args {
                use super::*;
                pub mod mod_id {
                    use super::*;
                    pub const __NAME: &str = stringify!(id);
                    pub const __TY: &str = "UserId";
                    pub const __EXPR: &str = "UserId::new()";
                    #[inline]
                    pub fn value() -> UserId {
                        UserId::new()
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(my_step1::id)
                    );
                }
                pub mod mod_name {
                    use super::*;
                    pub const __NAME: &str = stringify!(name);
                    pub const __TY: &str = "&str";
                    pub const __EXPR: &str = "\"Bob\"";
                    #[inline]
                    pub fn value() -> &'static str {
                        "Bob"
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(my_step1::name)
                    );
                }
            }
            impl my_step1 {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__NAME,
                        Self::name => my_step1_args::mod_name::__NAME,
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__TY,
                        Self::name => my_step1_args::mod_name::__TY,
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::id => my_step1_args::mod_id::__EXPR,
                        Self::name => my_step1_args::mod_name::__EXPR,
                    }
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    match self {
                        Self::id => ArgValue::my_step1(arg_values::my_step1::id(my_step1_args::mod_id::value())),
                        Self::name => ArgValue::my_step1(arg_values::my_step1::name(my_step1_args::mod_name::value())),
                    }
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    match self {
                        Self::id => my_step1_args::mod_id::DYN_STEP_ARG,
                        Self::name => my_step1_args::mod_name::DYN_STEP_ARG,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_arg_values() {
        let step: StoryStep = parse_quote! {
            #[step("Step 1")]
            fn my_step1(id: UserId, name: &str);
        };
        let actual = generate_arg_values(&step);
        let expected = quote! {
            #[allow(non_camel_case_types)]
            #[derive(narrative::serde::Serialize)]
            #[serde(untagged, crate = "narrative::serde")]
            pub(super) enum my_step1 {
                id(UserId),
                name(&'static str),
            }

            impl std::fmt::Debug for my_step1 {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    match self {
                        Self::id(arg) => arg.fmt(f),
                        Self::name(arg) => arg.fmt(f),
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_const_usage() {
        let step: StoryStep = parse_quote! {
            #[step("Step with const: {val}", val = MY_CONST * 2)]
            fn step_with_const(val: i32);
        };
        let story_syntax: ItemStory = syn::parse_quote! {
            trait ConstStory {
                const MY_CONST: i32 = 10;
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod step_with_const_args {
                use super::*;
                pub mod mod_val {
                    use super::*;
                    pub const __NAME: &str = stringify!(val);
                    pub const __TY: &str = "i32";
                    pub const __EXPR: &str = "MY_CONST * 2";
                    #[inline]
                    pub fn value() -> i32 {
                        MY_CONST * 2
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(step_with_const::val)
                    );
                }
            }
            impl step_with_const {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__NAME,
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__TY,
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__EXPR,
                    }
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    match self {
                        Self::val => ArgValue::step_with_const(arg_values::step_with_const::val(step_with_const_args::mod_val::value())),
                    }
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    match self {
                        Self::val => step_with_const_args::mod_val::DYN_STEP_ARG,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_const_usage_with_format() {
        let step: StoryStep = parse_quote! {
            #[step("Step with const: {val}", val = format!("const: {MY_CONST}"))]
            fn step_with_const(val: String);
        };
        let story_syntax: ItemStory = syn::parse_quote! {
            trait ConstStory {
                const MY_CONST: i32 = 10;
                #step
            }
        };
        let actual = generate_arg_impl(&story_syntax, &step);
        let expected = quote! {
            pub mod step_with_const_args {
                use super::*;
                pub mod mod_val {
                    use super::*;
                    pub const __NAME: &str = stringify!(val);
                    pub const __TY: &str = "String";
                    pub const __EXPR: &str = "format!(\"const: {MY_CONST}\")";
                    #[inline]
                    pub fn value() -> String {
                        format!("const: {MY_CONST}")
                    }
                    pub const DYN_STEP_ARG: narrative::step::DynStepArg = narrative::step::DynStepArg::new(
                        __NAME,
                        __TY,
                        __EXPR,
                        || narrative::value::BoxedValue::new(value()),
                        || narrative::value::BoxedValue::new(step_with_const::val)
                    );
                }
            }
            impl step_with_const {
                #[inline]
                pub(super) fn name(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__NAME,
                    }
                }
                #[inline]
                pub(super) fn ty(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__TY,
                    }
                }
                #[inline]
                pub(super) fn expr(&self) -> &'static str {
                    match self {
                        Self::val => step_with_const_args::mod_val::__EXPR,
                    }
                }
                #[inline]
                pub(super) fn value(&self) -> ArgValue {
                    match self {
                        Self::val => ArgValue::step_with_const(arg_values::step_with_const::val(step_with_const_args::mod_val::value())),
                    }
                }
                #[inline]
                pub(super) fn to_dyn(&self) -> narrative::step::DynStepArg {
                    match self {
                        Self::val => step_with_const_args::mod_val::DYN_STEP_ARG,
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
