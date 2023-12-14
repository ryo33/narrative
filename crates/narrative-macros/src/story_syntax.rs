use syn::{
    braced,
    parse::{Parse, ParseStream},
    Token,
};

pub struct ItemStory {
    pub trait_token: Token![trait],
    pub ident: syn::Ident,
    pub brace_token: syn::token::Brace,
    pub items: Vec<StoryItem>,
}

pub enum StoryItem {
    Step(syn::TraitItemFn),
    Trait(syn::ItemTrait),
    Struct(syn::ItemStruct),
    Enum(syn::ItemEnum),
    Let(syn::ExprLet),
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

impl Parse for StoryItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(step) = input.parse().map(Self::Step) {
            Ok(step)
        } else if let Ok(trait_) = input.parse().map(Self::Trait) {
            Ok(trait_)
        } else if let Ok(struct_) = input.parse().map(Self::Struct) {
            Ok(struct_)
        } else if let Ok(enum_) = input.parse().map(Self::Enum) {
            Ok(enum_)
        } else if let Ok(let_) = input.parse().map(Self::Let) {
            let _ = input.parse::<Token![;]>()?;
            Ok(let_)
        } else {
            // I want to return more helpful error by looking ahead some tokens.
            Err(input.error("expected a step, trait, struct, enum, or let"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_step() {
        let input = quote! {
            #[step("Hi, I'm a user")]
            fn as_a_user();
        };
        let StoryItem::Step(step) = syn::parse2(input).unwrap() else {
            panic!("Expected a step");
        };
        assert_eq!(step.sig.ident, "as_a_user");
    }

    #[test]
    fn parse_trait() {
        let input = quote! {
            trait UserId {
                fn new_v4() -> Self;
            }
        };
        let StoryItem::Trait(trait_) = syn::parse2(input).unwrap() else {
            panic!("Expected a trait");
        };
        assert_eq!(trait_.ident, "UserId");
    }

    #[test]
    fn parse_struct() {
        let input = quote! {
            struct UserName(String);
        };
        let StoryItem::Struct(struct_) = syn::parse2(input).unwrap() else {
            panic!("Expected a struct");
        };
        assert_eq!(struct_.ident, "UserName");
    }

    #[test]
    fn parse_enum() {
        let input = quote! {
            enum UserKind {
                Admin,
                Developer,
                Normal,
            }
        };
        let StoryItem::Enum(enum_) = syn::parse2(input).unwrap() else {
            panic!("Expected an enum");
        };
        assert_eq!(enum_.ident, "UserKind");
    }

    #[test]
    fn parse_let() {
        let input = quote! {
            let user_id = UserId::new_v4();
        };
        let StoryItem::Let(let_) = syn::parse2(input).unwrap() else {
            panic!("Expected a let");
        };
        let syn::Pat::Ident(ident) = *let_.pat else {
            panic!("Expected an ident");
        };
        assert_eq!(ident.ident.to_string(), "user_id");
    }

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
