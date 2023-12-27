mod generate;

use darling::FromDeriveInput;
use generate::generate;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(template))]
struct TemplateOptions {
    ident: Ident,
    generics: syn::Generics,
    name: String,
    #[darling(default)]
    mime: Option<String>,
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = TemplateOptions::from_derive_input(&input).expect("failed to parse input");
    generate(options).into()
}
