use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(step);
}

pub struct StepAttr {
    pub pound_symbol: syn::Token![#],
    pub bracket: syn::token::Bracket,
    pub step: kw::step,
    pub paren: syn::token::Paren,
    pub text: syn::LitStr,
    pub args: Vec<StepAttrArgs>,
}

pub struct StepAttrArgs {
    pub comma_token: Option<syn::Token![,]>,
    pub ident: syn::Ident,
    pub equal_token: syn::Token![=],
    pub value: syn::Expr,
}

impl Parse for StepAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pound_symbol = input.parse::<syn::Token![#]>()?;
        let attr_content;
        let bracket = syn::bracketed!(attr_content in input);
        let step = attr_content.parse::<kw::step>()?;
        let step_content;
        let paren = syn::parenthesized!(step_content in attr_content);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_attr() {
        let input: StepAttr = syn::parse_quote! {
            #[step("Hello, world!")]
        };
        assert_eq!(input.text.value(), "Hello, world!".to_string());
        assert_eq!(input.args.len(), 0);
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
}
