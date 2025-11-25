pub mod story_const;
pub mod story_item;
pub mod story_step;

use syn::{
    Token, braced,
    parse::{Parse, ParseStream},
};

pub use story_item::StoryItem;
pub use story_step::StoryStep;

use self::story_const::StoryConst;

pub struct ItemStory {
    pub attrs: Vec<syn::Attribute>,
    #[allow(dead_code)]
    pub trait_token: Token![trait],
    pub ident: syn::Ident,
    #[allow(dead_code)]
    pub brace_token: syn::token::Brace,
    pub items: Vec<StoryItem>,
}

impl Parse for ItemStory {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let story_token = input.parse::<Token![trait]>()?;
        let ident = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            attrs,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_story() {
        let input = quote! {
            trait MyFirstStory {
                const user_id: String = "test-id".to_string();

                #[step("Hi, I'm a user")]
                fn as_a_user();
                #[step("I have an apple", count = 1)]
                fn have_one_apple(count: u32);
            }
        };
        let ItemStory {
            attrs: _,
            trait_token: _,
            ident,
            brace_token: _,
            items,
        } = syn::parse2(input).expect("parse a story");
        assert_eq!(ident, "MyFirstStory");
        assert_eq!(items.len(), 3);
        assert!(matches!(items[0], StoryItem::Const { .. }));
        assert!(matches!(items[1], StoryItem::Step(_)));
        assert!(matches!(items[2], StoryItem::Step(_)));
    }

    #[test]
    fn parse_story_with_doc_attr() {
        let input = quote! {
            /// This is a my first story.
            trait MyFirstStory {
                const user_id: String = "test-id".to_string();

                #[step("Hi, I'm a user")]
                fn as_a_user();
            }
        };
        let ItemStory {
            attrs,
            ident,
            items,
            ..
        } = syn::parse2(input).expect("parse a story with doc attr");
        assert_eq!(ident, "MyFirstStory");
        assert_eq!(attrs.len(), 1);
        assert!(attrs[0].path().is_ident("doc"));
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], StoryItem::Const { .. }));
        assert!(matches!(items[1], StoryItem::Step(_)));
    }
}
