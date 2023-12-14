use syn::parse::{Parse, ParseStream};

// TODO:
pub struct StoryAttr;

impl Parse for StoryAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Expr>()?;
        Ok(Self)
    }
}
