mod derive_attr;
mod enum_def;
mod struct_def;
mod trait_def;

// Separation of impls and assertions leads not only simple implementation but also good error
// messages that indicate an outer dependency.
mod local_type_assertions;
mod local_type_impls;

mod base_trait;
mod step_fn;
mod story_trait;

mod step_args;
mod step_types;
mod story_consts;
mod story_context;
mod story_ext;

mod dummy_environment;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    item_story::{ItemStory, StoryItem},
    story_attr_syntax::StoryAttr,
    Asyncness,
};

pub(crate) fn generate(attr: &StoryAttr, item: &ItemStory) -> TokenStream {
    let mod_ident = format_ident!("mod_{}", item.ident);
    let ident = &item.ident;
    let async_ident = format_ident!("Async{}", item.ident);
    let context_ident = format_ident!("{}Context", item.ident);
    let base_trait = base_trait::generate(item, Asyncness::Sync);
    let async_base_trait = base_trait::generate(item, Asyncness::Async);
    let story_trait = story_trait::generate(item, Asyncness::Sync);
    let async_story_trait = story_trait::generate(item, Asyncness::Async);
    let step_args = step_args::generate(item);
    let step_types = step_types::generate(item);
    let story_consts = story_consts::generate(item);
    let story_context = story_context::generate(attr, item);
    let context_ext = story_context::generate_ext(item);
    let story_ext = story_ext::generate(item, Asyncness::Sync);
    let async_story_ext = story_ext::generate(item, Asyncness::Async);
    let local_type_impls = local_type_impls::generate(item);
    let local_type_assertions = local_type_assertions::generate(item);
    let dummy_environment = dummy_environment::generate(item, Asyncness::Sync);
    let async_dummy_environment = dummy_environment::generate(item, Asyncness::Async);
    let definitions = item.items.iter().filter_map(|item| match item {
        StoryItem::Struct(item) => Some(struct_def::generate(item)),
        StoryItem::Enum(item) => Some(enum_def::generate(item)),
        StoryItem::Trait(item) => Some(trait_def::generate(item)),
        _ => None,
    });
    quote! {
        #[allow(non_snake_case)]
        mod #mod_ident {
            #base_trait
            #async_base_trait
            #story_trait
            #async_story_trait
            #step_args
            #step_types
            #story_consts
            #story_context
            #context_ext
            #story_ext
            #async_story_ext
            #local_type_impls
            #local_type_assertions
            #dummy_environment
            #async_dummy_environment
            #(#definitions)*
        }
        #[allow(unused_imports)]
        use narrative::prelude::*;
        #[allow(unused_imports)]
        pub use #mod_ident::#ident;
        #[allow(unused_imports)]
        pub use #mod_ident::#async_ident;
        #[allow(unused_imports)]
        pub use #mod_ident::StoryContext as #context_ident;
        pub use #mod_ident::StoryExt as _;
        pub use #mod_ident::AsyncStoryExt as _;
        pub use #mod_ident::ContextExt as _;
        pub use #mod_ident::AsyncContextExt as _;
        pub use #mod_ident::BaseTrait as _;
    }
}

struct MatchArms(Vec<TokenStream>);

impl FromIterator<TokenStream> for MatchArms {
    fn from_iter<T: IntoIterator<Item = TokenStream>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a TokenStream> for MatchArms {
    fn from_iter<T: IntoIterator<Item = &'a TokenStream>>(iter: T) -> Self {
        Self(iter.into_iter().cloned().collect())
    }
}

impl ToTokens for MatchArms {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let extend = if self.0.is_empty() {
            quote! {
                unreachable!()
            }
        } else {
            let arms = &self.0;
            quote! {
                match self {
                    #(#arms)*
                }
            }
        };
        tokens.extend(::core::iter::once(extend));
    }
}
