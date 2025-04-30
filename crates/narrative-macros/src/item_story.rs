pub mod story_const;
pub mod story_item;
pub mod story_step;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
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
            known_consts: &'a HashSet<String>,
            used_consts: HashSet<String>,
        }

        impl<'a> ConstUsageVisitor<'a> {
            fn new(known_consts: &'a HashSet<String>) -> Self {
                Self {
                    known_consts,
                    used_consts: HashSet::new(),
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
                if mac.path.is_ident("format") || mac.path.is_ident("format_args") {
                    let only_first_token = mac.tokens.clone().into_iter().take(1).collect();
                    let format: syn::LitStr = syn::parse2(only_first_token).unwrap();
                    for arg in collect_format_args(&format) {
                        if self.known_consts.contains(&arg) {
                            self.used_consts.insert(arg);
                        }
                    }
                }
            }
        }

        // Find used constants in the expression
        let const_idents: HashSet<String> =
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

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
}
