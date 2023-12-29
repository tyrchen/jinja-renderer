mod generate;

use darling::FromDeriveInput;
use generate::{generate_event_trait, generate_render_context_trait};
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

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(event))]
struct EventOptions {
    ident: Ident,
    generics: syn::Generics,
    name: String,
    receivers: String,
    target: String,
    #[darling(default = "default_swap")]
    swap: String,
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = TemplateOptions::from_derive_input(&input).expect("failed to parse input");
    generate_render_context_trait(options).into()
}

#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = EventOptions::from_derive_input(&input).expect("failed to parse input");
    generate_event_trait(options).into()
}

fn default_swap() -> String {
    "innerHTML".to_string()
}
