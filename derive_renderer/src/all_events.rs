use crate::{event::get_enum_data_types, AllEventsOptions};
// only proc_macro2::TokenStream is testable
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate_all_events_fn(options: AllEventsOptions) -> TokenStream {
    let ident = options.ident;
    let generics = options.generics;
    let vis = options.vis;
    let data = match options.data {
        darling::ast::Data::Enum(v) => get_enum_data_types(v)
            .map(|v| quote! { #v::EVENT_NAME })
            .collect::<Vec<_>>(),
        _ => panic!("AllEvents only support enum"),
    };

    quote! {
        impl #generics #ident #generics {
            #vis fn all_events() -> &'static [&'static str] {
              &[#(#data),*]
            }

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::FromDeriveInput;
    use jinja_renderer::Event;
    use quote::quote;
    use serde::Serialize;
    use syn::parse_quote;

    #[derive(Debug, Serialize, Event)]
    #[event(name = "foo", receivers = "#foo", target = "#foo")]
    struct Foo1<'a> {
        bar: &'a str,
    }

    #[derive(Debug, Serialize, Event)]
    #[event(name = "bar", receivers = "#bar", target = "#bar")]
    struct Bar1<'a> {
        bar: &'a str,
    }

    #[derive(Debug, Serialize, Event)]
    #[event(name = "baz", receivers = "#baz", target = "dynamic")]
    struct Baz1<'a> {
        bar: &'a str,
        id: &'a str,
    }

    #[test]
    fn all_events_should_work() {
        let input = parse_quote! {


            #[derive(Debug, AllEvents)]
            pub(crate) enum AllEvents<'a> {
                Foo2(Foo1<'a>),
                Bar2(Bar1<'a>),
                Baz2(Baz1<'a>),
            }
        };
        let options = AllEventsOptions::from_derive_input(&input).unwrap();
        let expected = quote! {
            impl<'a> AllEvents<'a> {
                pub(crate) fn all_events() -> &'static [&'static str] {
                    &[Foo1::EVENT_NAME, Bar1::EVENT_NAME, Baz1::EVENT_NAME]
                }
            }
        };
        let actual = generate_all_events_fn(options).to_string();
        assert_eq!(actual, expected.to_string());
    }
}
