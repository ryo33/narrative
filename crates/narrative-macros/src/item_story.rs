pub mod story_item;
pub mod story_step;

use syn::{
    braced,
    parse::{Parse, ParseStream},
    Token,
};

pub use story_item::StoryItem;
pub use story_step::StoryStep;

pub struct ItemStory {
    pub trait_token: Token![trait],
    pub ident: syn::Ident,
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
    pub(crate) fn steps(&self) -> impl Iterator<Item = &StoryStep> {
        self.items.iter().filter_map(|item| match item {
            StoryItem::Step(step) => Some(step),
            _ => None,
        })
    }
    pub(crate) fn assignments(&self) -> impl Iterator<Item = (&syn::Ident, &syn::Expr)> {
        self.items.iter().filter_map(|item| match item {
            StoryItem::Let(assignment) => {
                if let syn::Pat::Ident(pat_ident) = assignment.pat.as_ref() {
                    Some((&pat_ident.ident, assignment.expr.as_ref()))
                } else {
                    None
                }
            }
            _ => None,
        })
    }
    pub(crate) fn find_assignments<'a>(&'a self, ident: &'a syn::Ident) -> Option<&'a syn::Expr> {
        self.assignments()
            .find_map(move |(name, expr)| if name == ident { Some(expr) } else { None })
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
                struct UserName(String);
                enum UserKind {
                    Admin,
                    Developer,
                    Normal,
                }
                trait UserId {
                    fn new_v4() -> Self;
                }
                let user_id = UserId::new_v4();

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
        assert_eq!(items.len(), 6);
        assert!(matches!(items[0], StoryItem::Struct(_)));
        assert!(matches!(items[1], StoryItem::Enum(_)));
        assert!(matches!(items[2], StoryItem::Trait(_)));
        assert!(matches!(items[3], StoryItem::Let(_)));
        assert!(matches!(items[4], StoryItem::Step(_)));
        assert!(matches!(items[5], StoryItem::Step(_)));
    }
}
