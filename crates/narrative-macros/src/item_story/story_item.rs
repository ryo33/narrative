use syn::parse::{Parse, ParseStream};

use super::{story_const::StoryConst, StoryStep};

#[expect(clippy::large_enum_variant)]
pub enum StoryItem {
    Step(StoryStep),
    Trait(syn::ItemTrait),
    Struct(syn::ItemStruct),
    Enum(syn::ItemEnum),
    Const(StoryConst),
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
        } else if let Ok(const_) = input.parse::<syn::TraitItemConst>() {
            let Some(default) = const_.default.clone() else {
                return Err(input.error("in a story, all consts must have a value"));
            };
            Ok(Self::Const(StoryConst {
                raw: const_,
                default,
            }))
        } else {
            // I want to return more helpful error by looking ahead some tokens.
            Err(input.error("expected a step, trait, struct, enum, or let"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
        assert_eq!(step.inner.sig.ident, "as_a_user");
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
    fn parse_const() {
        let input = quote! {
            const user_id: UserId = UserId::new_v4();
        };
        let StoryItem::Const(StoryConst { raw: const_, .. }) = syn::parse2(input).unwrap() else {
            panic!("Expected a const");
        };
        assert_eq!(const_.ident.to_string(), "user_id");
    }
}
