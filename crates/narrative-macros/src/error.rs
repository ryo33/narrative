use proc_macro2::TokenStream;

#[derive(Debug)]
pub(crate) enum Error {
    PatNotSupported(syn::Pat),
}

impl Error {
    pub(crate) fn to_compile_error(self) -> TokenStream {
        match self {
            Self::PatNotSupported(pat) => {
                todo!()
            }
        }
    }
}
