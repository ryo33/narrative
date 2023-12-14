mod enum_def;
mod step_fn;
mod step_iter;
mod story_ext;
mod struct_def;
mod type_def;

use proc_macro2::TokenStream;

use crate::story_syntax::ItemStory;

pub fn output(story_syntax: ItemStory) -> TokenStream {
    todo!()
}

#[cfg(test)]
mod tests {}
