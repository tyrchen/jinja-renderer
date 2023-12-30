mod all_events;
mod context;
mod event;

use all_events::generate_all_events_fn;
use context::generate_render_context_trait;
use darling::{FromDeriveInput, FromField, FromVariant};
use event::generate_event_trait;
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
#[darling(attributes(event), forward_attrs(template))]
struct EventOptions {
    ident: Ident,
    generics: syn::Generics,
    #[darling(default)]
    name: String,
    #[darling(default)]
    receivers: String,
    #[darling(default = "default_target")]
    target: String,
    #[darling(default = "default_swap")]
    swap: String,
    #[darling(default = "default_id_field")]
    id_field: String,
    #[darling(default = "default_id_prefix")]
    id_prefix: String,
    data: darling::ast::Data<EnumData, FieldData>,
    attrs: Vec<syn::Attribute>,
}

#[derive(Debug, FromDeriveInput)]
struct AllEventsOptions {
    ident: Ident,
    generics: syn::Generics,
    vis: syn::Visibility,
    data: darling::ast::Data<EnumData, darling::util::Ignored>,
}

#[derive(Debug, FromField)]
struct FieldData {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

#[derive(Debug, FromVariant)]
struct EnumData {
    ident: syn::Ident,
    fields: darling::ast::Fields<FieldData>,
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

#[proc_macro_derive(AllEvents)]
pub fn derive_all_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = AllEventsOptions::from_derive_input(&input).expect("failed to parse input");
    generate_all_events_fn(options).into()
}

fn default_swap() -> String {
    "innerHTML".to_string()
}

fn default_target() -> String {
    "dynamic".to_string()
}

fn default_id_field() -> String {
    "id".to_string()
}

fn default_id_prefix() -> String {
    "id-".to_string()
}
