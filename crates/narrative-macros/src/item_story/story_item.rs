use syn::parse::{Parse, ParseStream};

use super::{story_const::StoryConst, StoryStep};

#[allow(clippy::large_enum_variant)]
pub enum StoryItem {
    Step(StoryStep),
    Const(StoryConst),
}

impl Parse for StoryItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(step) = input.parse().map(Self::Step) {
            Ok(step)
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
            Err(input.error("expected a step or const"))
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
