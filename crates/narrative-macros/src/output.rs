mod derive_attr;
mod enum_def;
mod struct_def;
mod trait_def;

// Separation of impls and assertions leads not only simple implementation but also good error
// messages that indicate an outer dependency.
mod local_type_assertions;
mod local_type_impls;

mod unused_assignments;

mod base_trait;
mod step_fn;
mod story_trait;

mod dummy_environment;
mod step_types;
mod story_context;
mod story_ext;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    item_story::{ItemStory, StoryItem},
    story_attr_syntax::StoryAttr,
    Asyncness,
};

pub fn generate(attr: &StoryAttr, item: &ItemStory) -> TokenStream {
    let base_trait = base_trait::generate(item);
    let story_trait = story_trait::generate(item, Asyncness::Sync);
    let step_types = step_types::generate(item, Asyncness::Sync);
    let story_context = story_context::generate(item, Asyncness::Sync);
    let story_ext = story_ext::generate(attr, item, Asyncness::Sync);
    let local_type_impls = local_type_impls::generate(item);
    let local_type_assertions = local_type_assertions::generate(item);
    let unused_assignments = unused_assignments::generate(item);
    let dummy_environment = dummy_environment::generate(item);
    let definitions = item.items.iter().filter_map(|item| match item {
        StoryItem::Struct(item) => Some(struct_def::generate(item)),
        StoryItem::Enum(item) => Some(enum_def::generate(item)),
        StoryItem::Trait(item) => Some(trait_def::generate(item)),
        _ => None,
    });
    quote! {
        #base_trait
        #story_trait
        #step_types
        #story_context
        #story_ext
        #local_type_impls
        #local_type_assertions
        #unused_assignments
        #dummy_environment
        #(#definitions)*
    }
}
