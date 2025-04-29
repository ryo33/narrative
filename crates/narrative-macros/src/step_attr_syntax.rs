use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(step);
    syn::custom_keyword!(story);
}

pub struct StepAttr {
    pub pound_symbol: syn::Token![#],
    pub bracket: syn::token::Bracket,
    pub step: kw::step,
    pub paren: syn::token::Paren,
    pub story_type: Option<StoryType>,
    pub text: syn::LitStr,
    pub args: Vec<StepAttrArgs>,
}

pub struct StoryType {
    pub story_kw: kw::story,
    pub colon_token: syn::Token![:],
    pub path: syn::Path,
    pub comma_token: syn::Token![,],
}

pub struct StepAttrArgs {
    pub comma_token: Option<syn::Token![,]>,
    pub ident: syn::Ident,
    pub equal_token: syn::Token![=],
    pub value: syn::Expr,
}

impl StoryType {
    pub fn path(&self) -> &syn::Path {
        &self.path
    }

    pub fn async_path(&self) -> syn::Path {
        let mut cloned = self.path.clone();
        if let Some(seg) = cloned.segments.last_mut() {
            seg.ident = format_ident!("Async{}", seg.ident);
        }
        cloned
    }

    pub fn context_path(&self) -> syn::Path {
        let mut cloned = self.path.clone();
        if let Some(seg) = cloned.segments.last_mut() {
            seg.ident = format_ident!("{}Context", seg.ident);
        }
        cloned
    }
}

impl Parse for StepAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pound_symbol = input.parse::<syn::Token![#]>()?;
        let attr_content;
        let bracket = syn::bracketed!(attr_content in input);
        let step = attr_content.parse::<kw::step>()?;
        let step_content;
        let paren = syn::parenthesized!(step_content in attr_content);

        // Try to parse a story type if available
        let story_type = if step_content.peek(kw::story) {
            Some(StoryType {
                story_kw: step_content.parse()?,
                colon_token: step_content.parse()?,
                path: step_content.parse()?,
                comma_token: step_content.parse()?,
            })
        } else {
            None
        };

        let text = step_content.parse()?;
        let mut args = Vec::new();
        while !step_content.is_empty() {
            args.push(step_content.parse()?);
        }

        Ok(Self {
            pound_symbol,
            bracket,
            step,
            paren,
            story_type,
            text,
            args,
        })
    }
}

impl Parse for StepAttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comma_token = input.parse::<Option<syn::Token![,]>>()?;
        let ident = input.parse::<syn::Ident>()?;
        let equal_token = input.parse::<syn::Token![=]>()?;
        let value = input.parse::<syn::Expr>()?;
        Ok(Self {
            comma_token,
            ident,
            equal_token,
            value,
        })
    }
}

impl ToTokens for StepAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pound_symbol.to_tokens(tokens);
        self.bracket.surround(tokens, |tokens| {
            self.step.to_tokens(tokens);
            self.paren.surround(tokens, |tokens| {
                if let Some(story_type) = &self.story_type {
                    story_type.to_tokens(tokens);
                }
                self.text.to_tokens(tokens);
                for arg in &self.args {
                    arg.to_tokens(tokens);
                }
            });
        });
    }
}

impl ToTokens for StoryType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.story_kw.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.path.to_tokens(tokens);
        self.comma_token.to_tokens(tokens);
    }
}

impl ToTokens for StepAttrArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.comma_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.equal_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_step_attr() {
        let input: StepAttr = syn::parse_quote! {
            #[step("Hello, world!")]
        };
        assert_eq!(input.text.value(), "Hello, world!".to_string());
        assert_eq!(input.args.len(), 0);
        assert!(input.story_type.is_none());
    }

    #[test]
    fn test_step_attr_with_args() {
        let input: StepAttr = syn::parse_quote! {
            #[step("Hello, world!", arg1 = 1, arg2 = "2", arg3 = UserId::new_v4())]
        };
        assert_eq!(input.text.value(), "Hello, world!".to_string());
        assert_eq!(input.args.len(), 3);
        assert_eq!(input.args[0].ident, "arg1".to_string());
        assert_eq!(input.args[1].ident, "arg2".to_string());
        assert_eq!(input.args[2].ident, "arg3".to_string());
        assert!(matches!(
            &input.args[0].value,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(value),
                ..
            }) if value.to_string() == "1"
        ));
        assert!(matches!(
            &input.args[1].value,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(value),
                ..
            }) if value.value() == "2"
        ));
        assert!(matches!(
            &input.args[2].value,
            syn::Expr::Call(syn::ExprCall { func, .. }) if matches!(func.as_ref(), syn::Expr::Path(syn::ExprPath { path, .. }) if path.segments.len() == 2 && path.segments[0].ident == "UserId")
        ));
    }

    #[test]
    fn test_step_attr_with_story() {
        let input: StepAttr = syn::parse_quote! {
            #[step(story: SubStory, "do sub story")]
        };
        assert_eq!(input.text.value(), "do sub story".to_string());
        assert_eq!(input.args.len(), 0);
        assert!(input.story_type.is_some());
        let story_type = input.story_type.as_ref().unwrap();
        assert!(story_type.path.is_ident("SubStory"));
    }

    #[test]
    fn test_step_attr_with_story_and_args() {
        let input: StepAttr = syn::parse_quote! {
            #[step(story: SubStory, "do sub story with args", arg = 2)]
        };
        assert_eq!(input.text.value(), "do sub story with args".to_string());
        assert_eq!(input.args.len(), 1);
        assert_eq!(input.args[0].ident, "arg".to_string());
        assert!(input.story_type.is_some());
        let story_type = input.story_type.as_ref().unwrap();
        assert!(story_type.path.is_ident("SubStory"));
    }

    #[test]
    fn test_to_tokens() {
        let input: StepAttr = syn::parse_quote! {
            #[step("Hello, world!", arg1 = 1, arg2 = "2", arg3 = UserId::new_v4())]
        };
        let actual = quote! {
            #input
        };
        let expected = quote! {
            #[step("Hello, world!", arg1 = 1, arg2 = "2", arg3 = UserId::new_v4())]
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_to_tokens_with_story() {
        let input: StepAttr = syn::parse_quote! {
            #[step(story: SubStory, "do sub story", arg = 2)]
        };
        let actual = quote! {
            #input
        };
        let expected = quote! {
            #[step(story: SubStory, "do sub story", arg = 2)]
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
