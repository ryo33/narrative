use proc_macro2::TokenStream;
use quote::quote;

pub fn generate() -> TokenStream {
    // Clone, Debug, PartialEq are primitive traits for structured data in general.
    // Eq cannot be derived for floating point numbers.
    // PartialOrd and Hash cannot be derived for HashMap
    // Serialize is for transfer arguments to other processes.
    quote!(#[derive(Clone, Debug, PartialEq, narrative::Serialize)])
}
