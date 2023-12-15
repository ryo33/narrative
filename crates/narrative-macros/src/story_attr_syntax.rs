use syn::{
    bracketed,
    parse::{Parse, ParseStream},
};

mod kw {
    syn::custom_keyword!(story);
}

pub struct StoryAttr {
    title: syn::LitStr,
}

impl Parse for StoryAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let title = input.parse()?;
        Ok(Self { title })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_attr() {
        let input: StoryAttr = syn::parse_quote! {
            "Hello, world!"
        };
        assert_eq!(input.title.value(), "Hello, world!".to_string());
    }
}
