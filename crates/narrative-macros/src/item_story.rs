pub mod story_item;
pub mod story_step;

use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote_spanned, Token,
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
    pub(crate) fn find_assignments<'a>(&'a self, ident: &'a syn::Ident) -> Option<&'a syn::Expr> {
        self.items.iter().find_map(|item| match item {
            StoryItem::Const { raw, default } => {
                if raw.ident == *ident {
                    Some(&default.1)
                } else {
                    None
                }
            }
            _ => None,
        })
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
