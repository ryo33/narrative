mod error;
mod extract_types_for_assertion;
mod item_story;
mod local_type_for;
mod no_foreign_type_validation;
mod output;
mod step_attr_syntax;
mod step_usage;
mod story_attr_syntax;

use item_story::ItemStory;
use proc_macro2::TokenStream;
use story_attr_syntax::StoryAttr;
use syn::parse_macro_input;

#[proc_macro_attribute]
/// TODO: Add documentation.
pub fn story(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as StoryAttr);
    let story = parse_macro_input!(input as ItemStory);
    process_story(attr, story).into()
}

#[proc_macro_attribute]
/// Marks a data type as a local type for a specific story.
/// This implements both `IndependentType` and `<StoryName>LocalType` for the type.
pub fn local_type_for(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let story_name = parse_macro_input!(attr as syn::Ident);
    let input_item = parse_macro_input!(input as syn::Item);

    local_type_for::generate(&story_name, &input_item).into()
}

// In general, we don't do caching some intermediate results to keep the implementation simple.
// However, we should avoid to have heavy computation in this crate, to keep the story compilation
// fast. So, modules have their own functionality which is simple.
fn process_story(attr: StoryAttr, story: ItemStory) -> TokenStream {
    output::generate(&attr, &story)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Asyncness {
    Sync,
    Async,
}

impl quote::ToTokens for Asyncness {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Asyncness::Sync => quote::quote!().to_tokens(tokens),
            Asyncness::Async => quote::quote!(async).to_tokens(tokens),
        }
    }
}

pub(crate) fn collect_format_args(lit_str: &syn::LitStr) -> Vec<String> {
    lit_str
        .value()
        // remove escaped braces
        .split("{{")
        .flat_map(|part| part.split("}}"))
        // iter parts that start with '{' by skipping the first split
        .flat_map(|part| part.split('{').skip(1))
        // take the part before the first '}'
        .filter_map(|part| part.split_once('}').map(|(head, _)| head))
        // remove parts after the first ':'
        .map(|format| {
            format
                .split_once(':')
                .map(|(head, _)| head)
                .unwrap_or(format)
        })
        .map(ToOwned::to_owned)
        .collect()
}

struct MakeStaticWalker;

impl syn::visit_mut::VisitMut for MakeStaticWalker {
    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        i.lifetime = Some(syn::Lifetime::new(
            "'static",
            proc_macro2::Span::mixed_site(),
        ));
        self.visit_type_mut(&mut i.elem);
    }
}

pub(crate) fn make_static(ty: &syn::Type) -> syn::Type {
    use syn::visit_mut::VisitMut;
    let mut walker = MakeStaticWalker;
    let mut static_ty = ty.clone();
    walker.visit_type_mut(&mut static_ty);
    static_ty
}

pub(crate) fn pretty_print_expr(expr: &syn::Expr) -> String {
    prettyplease::unparse(
        &syn::parse_file(
            &quote::quote! {
                const IDENT: String = #expr;
            }
            .to_string(),
        )
        .unwrap(),
    )
    .replace("const IDENT: String = ", "")
    .replace(";", "")
    .trim()
    .to_string()
}

pub(crate) fn pretty_print_type(ty: &syn::Type) -> String {
    prettyplease::unparse(
        &syn::parse_file(
            &quote::quote! {
                const IDENT: #ty = 1;
            }
            .to_string(),
        )
        .unwrap(),
    )
    .replace("const IDENT: ", "")
    .replace(" = 1;", "")
    .trim()
    .to_string()
}
