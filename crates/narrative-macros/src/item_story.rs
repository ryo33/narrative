pub mod story_const;
pub mod story_item;
pub mod story_step;

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{discouraged::Speculative as _, Parse, ParseStream},
    punctuated::Punctuated,
    visit::Visit,
    Expr, Token,
};

pub use story_item::StoryItem;
pub use story_step::StoryStep;

use crate::collect_format_args;

use self::story_const::StoryConst;

pub struct ItemStory {
    #[allow(dead_code)]
    pub trait_token: Token![trait],
    pub ident: syn::Ident,
    #[allow(dead_code)]
    pub brace_token: syn::token::Brace,
    pub items: Vec<StoryItem>,
}

impl Parse for ItemStory {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let story_token = input.parse::<Token![trait]>()?;
        let ident = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            trait_token: story_token,
            ident,
            brace_token,
            items,
        })
    }
}

impl ItemStory {
    pub(crate) fn consts(&self) -> impl Iterator<Item = &StoryConst> {
        self.items.iter().filter_map(|item| match item {
            StoryItem::Const(item) => Some(item),
            _ => None,
        })
    }
    pub(crate) fn steps(&self) -> impl Iterator<Item = &StoryStep> {
        self.items.iter().filter_map(|item| match item {
            StoryItem::Step(step) => Some(step),
            _ => None,
        })
    }
    pub(crate) fn find_assignments<'a>(&'a self, ident: &'a syn::Ident) -> Option<&'a syn::Expr> {
        self.consts().find_map(|StoryConst { raw, default }| {
            if raw.ident == *ident {
                Some(&default.1)
            } else {
                None
            }
        })
    }
    /// This returns all the constants used in the story. Any where constants may be used can use this.
    /// rustc should optimize out unused ones.
    pub(crate) fn generate_const_bindings(
        &self,
        expr: &Expr,
    ) -> impl Iterator<Item = ConstBinding> {
        // Visitor to find constants used in an expression
        struct ConstUsageVisitor<'a> {
            known_consts: &'a BTreeSet<String>,
            used_consts: BTreeSet<String>,
        }

        impl<'a> ConstUsageVisitor<'a> {
            fn new(known_consts: &'a BTreeSet<String>) -> Self {
                Self {
                    known_consts,
                    used_consts: BTreeSet::new(),
                }
            }
        }

        impl<'ast> Visit<'ast> for ConstUsageVisitor<'_> {
            fn visit_path(&mut self, path: &'ast syn::Path) {
                if path.leading_colon.is_none() && path.segments.len() == 1 {
                    let ident = path.segments[0].ident.to_string();
                    if self.known_consts.contains(&ident) {
                        self.used_consts.insert(ident);
                    }
                }
            }

            fn visit_macro(&mut self, mac: &syn::Macro) {
                if mac.path.is_ident("format")
                    || mac.path.is_ident("format_args")
                    || mac.path.is_ident("println")
                    || mac.path.is_ident("eprintln")
                    || mac.path.is_ident("print")
                    || mac.path.is_ident("eprint")
                    || mac.path.is_ident("panic")
                    || mac.path.is_ident("todo")
                    || mac.path.is_ident("unimplemented")
                {
                    let only_first_token = mac.tokens.clone().into_iter().take(1).collect();
                    let format: syn::LitStr = syn::parse2(only_first_token).unwrap();
                    let mut extracted = collect_format_args(&format);
                    let Ok(args) = syn::parse2::<FormatArgs>(mac.tokens.clone()) else {
                        // This case format syntax should be wrong, so that rust-analyzer should report an error.
                        eprintln!("format syntax is wrong: {:?}", mac);
                        return;
                    };
                    for arg in args.0 {
                        match arg {
                            FormatArg::Var(expr) => self.visit_expr(&expr),
                            FormatArg::Named {
                                name,
                                _eq_token: _,
                                expr,
                            } => {
                                extracted.retain(|arg| name != arg);
                                self.visit_expr(&expr);
                            }
                        }
                    }
                    for arg in extracted {
                        if self.known_consts.contains(&arg) {
                            self.used_consts.insert(arg);
                        }
                    }
                }
            }
        }

        // Find used constants in the expression
        let const_idents: BTreeSet<String> =
            self.consts().map(|c| c.raw.ident.to_string()).collect();
        let mut visitor = ConstUsageVisitor::new(&const_idents);
        visitor.visit_expr(expr);

        self.consts()
            .filter(move |c| visitor.used_consts.contains(&c.raw.ident.to_string()))
            .map(|c| ConstBinding {
                ident: &c.raw.ident,
                ty: &c.raw.ty,
                expr: &c.default.1,
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstBinding<'a> {
    pub ident: &'a syn::Ident,
    pub ty: &'a syn::Type,
    pub expr: &'a syn::Expr,
}

impl ToTokens for ConstBinding<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let ty = &self.ty;
        let expr = &self.expr;
        tokens.extend(quote::quote! {
            let #ident: #ty = #expr;
        });
    }
}

struct FormatArgs(Punctuated<FormatArg, Token![,]>);
impl Parse for FormatArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<syn::LitStr>()?;
        let _ = input.parse::<syn::Token![,]>(); // no `?` here for the case with no args
        let args = Punctuated::parse_terminated(input)?;
        Ok(Self(args))
    }
}
enum FormatArg {
    Var(syn::Expr),
    Named {
        name: syn::Ident,
        _eq_token: syn::Token![=],
        expr: syn::Expr,
    },
}

impl Parse for FormatArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse_named(input: ParseStream) -> syn::Result<FormatArg> {
            let name = input.parse::<syn::Ident>()?;
            let eq_token = input.parse::<syn::Token![=]>()?;
            let expr = input.parse::<syn::Expr>()?;
            Ok(FormatArg::Named {
                name,
                _eq_token: eq_token,
                expr,
            })
        }
        let fork = input.fork();
        if let Ok(args) = parse_named(&fork) {
            input.advance_to(&fork);
            Ok(args)
        } else {
            Ok(FormatArg::Var(input.parse()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_format_arg() {
        let input: syn::Macro = syn::parse_quote! {
            format!("User: {}, Items: {unused}", user_id, unused = count)
        };
        let args = syn::parse2::<FormatArgs>(input.tokens).unwrap();
        assert_eq!(args.0.len(), 2);
    }

    #[test]
    fn parse_story() {
        let input = quote! {
            trait MyFirstStory {
                trait UserId {
                    fn new_v4() -> Self;
                }
                const user_id: UserId = UserId::new_v4();

                #[step("Hi, I'm a user")]
                fn as_a_user(user_id: UserId);
                #[step("I have an apple", count = 1)]
                fn have_one_apple(count: u32);
            }
        };
        let ItemStory {
            trait_token: _,
            ident,
            brace_token: _,
            items,
        } = syn::parse2(input).expect("parse a story");
        assert_eq!(ident, "MyFirstStory");
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], StoryItem::Trait(_)));
        assert!(matches!(items[1], StoryItem::Const { .. }));
        assert!(matches!(items[2], StoryItem::Step(_)));
        assert!(matches!(items[3], StoryItem::Step(_)));
    }

    fn create_test_story() -> ItemStory {
        let input = quote! {
            trait TestStory {
                const user_id: String = "test-user".to_string();
                const count: u32 = 5;
                const unused: bool = true;

                #[step("Test step")]
                fn test_step();
            }
        };
        syn::parse2::<ItemStory>(input).expect("Failed to parse story")
    }

    #[test]
    fn test_generate_const_bindings_single_constant() {
        let story = create_test_story();

        // Test with expression that uses one constant as a direct path reference
        let expr = syn::parse2::<Expr>(quote! {
            user_id
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].ident, "user_id");
    }

    #[test]
    fn test_generate_const_bindings_multiple_constants() {
        let story = create_test_story();

        // Test with expression that uses multiple constants in a tuple
        let expr = syn::parse2::<Expr>(quote! {
            (user_id, count)
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].ident, "user_id");
        assert_eq!(bindings[1].ident, "count");
    }

    #[test]
    fn test_generate_const_bindings_no_constants() {
        let story = create_test_story();

        // Test with expression that doesn't use any constants
        let expr = syn::parse2::<Expr>(quote! {
            "Hello world"
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_generate_const_bindings_format_macro() {
        let story = create_test_story();

        // Test with format macro
        let expr = syn::parse2::<Expr>(quote! {
            format!("User: {user_id}, Items: {count}")
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].ident, "user_id");
        assert_eq!(bindings[1].ident, "count");
    }

    #[test]
    fn test_generate_const_bindings_format_macro_with_args() {
        let story = create_test_story();

        // Test with format macro
        let expr = syn::parse2::<Expr>(quote! {
            format!("User: {}, Items: {unused}", user_id, unused = count)
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].ident, "user_id");
        assert_eq!(bindings[1].ident, "count");
    }

    #[test]
    fn test_generate_const_bindings_in_complex_expr() {
        let story = create_test_story();

        // Test with constants used in a more complex expression
        let expr = syn::parse2::<Expr>(quote! {
            if user_id.len() > 0 {
                count + 1
            } else {
                0
            }
        })
        .unwrap();

        let bindings: Vec<_> = story.generate_const_bindings(&expr).collect();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].ident, "user_id");
        assert_eq!(bindings[1].ident, "count");
    }
}
