pub struct StoryConst {
    pub raw: syn::TraitItemConst,
    pub default: (syn::Token![=], syn::Expr),
}
