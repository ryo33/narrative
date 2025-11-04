use proc_macro2::TokenStream;
use quote::quote;

pub struct StoryConst {
    pub raw: syn::TraitItemConst,
    pub default: (syn::Token![=], syn::Expr),
}

impl StoryConst {
    pub fn to_pub_const(&self) -> TokenStream {
        let syn::TraitItemConst {
            attrs,
            const_token,
            ident,
            generics,
            colon_token,
            ty,
            default: _,
            semi_token: _,
        } = &self.raw;
        let (eq, default) = self.default.clone();
        quote! {
            #(#attrs)*
            pub #const_token #ident #generics #colon_token #ty #eq #default;
        }
    }
}
