use crate::{EnumData, EventOptions, FieldData};
// only proc_macro2::TokenStream is testable
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub(crate) fn generate_event_trait(options: EventOptions) -> TokenStream {
    let EventOptions {
        ident,
        generics,
        name,
        receivers,
        target,
        swap,
        id_field,
        id_prefix,
        data,
        attrs,
    } = options;

    let receivers = receivers
        .trim()
        .split(' ')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    if name.is_empty() || receivers.is_empty() || target.is_empty() {
        match data {
            darling::ast::Data::Struct(_) => {
                panic!("event name, receivers and target must not be empty for struct")
            }
            darling::ast::Data::Enum(data) => {
                return generate_event_trait_for_enum(&ident, &generics, data);
            }
        }
    }

    // TODO: use regex to do more tests
    for receiver in &receivers {
        if !receiver.starts_with('#') {
            panic!("receivers must start with #, and separated by space");
        }
    }

    let target_code = if target == "dynamic" {
        let id_ident = Ident::new(&id_field, Span::call_site());
        if !check_id_exists(&data, &id_field) {
            panic!("target is dynamic, but {id_field} field not found");
        } else {
            quote! { format!("{}{}", #id_prefix, self.#id_ident).into() }
        }
    } else {
        quote! { #target.into() }
    };
    let event_info_code = quote! {
      fn event_info(&self) -> jinja_renderer::EventInfo {
        jinja_renderer::EventInfo {
          name: #name,
          receivers: &[#(#receivers),*],
          target: #target_code,
          swap: #swap,
          id_field: #id_field,
        }
      }
    };

    let event_info_with_data = quote! {
        serde_json::to_string(&serde_json::json!({"info": self.event_info(), "data": self})).expect("even info should be a valid json")
    };
    let render_event_code = if !attrs.is_empty() {
        // if template is specified, render template and attach it to event info
        quote! {
            let mut ret = #event_info_with_data;
            let data = self.render(renderer)?;
            ret.push_str("\n");
            ret.push_str(&data);
            Ok(ret)
        }
    } else {
        // otherwise, put the data into event info
        quote! {
          Ok(#event_info_with_data)
        }
    };

    quote! {
        impl #generics jinja_renderer::RenderEvent for #ident #generics {
            const EVENT_NAME: &'static str = #name;

            fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                #render_event_code
            }

            #event_info_code
        }


    }
}

pub(crate) fn get_enum_idents(data: Vec<EnumData>) -> impl Iterator<Item = Ident> {
    data.into_iter().map(|v| v.ident)
}

pub(crate) fn get_enum_data_types(data: Vec<EnumData>) -> impl Iterator<Item = TokenStream> {
    data.into_iter().map(|v| match v.fields.style {
        darling::ast::Style::Tuple => {
            if v.fields.fields.len() != 1 {
                panic!("only support tuple struct with one field");
            }
            let ty = &v.fields.fields[0].ty;
            let names = match ty {
                syn::Type::Path(p) => {
                    let path = &p.path;
                    path.segments.iter().map(|s| &s.ident).collect::<Vec<_>>()
                }
                _ => panic!("only support standard type path"),
            };
            quote! { #(#names)::* }
        }
        _ => panic!("only support tuple struct"),
    })
}

fn generate_event_trait_for_enum(
    ident: &Ident,
    generics: &syn::Generics,
    data: Vec<EnumData>,
) -> TokenStream {
    let data = &get_enum_idents(data).collect::<Vec<_>>();

    quote! {
        impl #generics jinja_renderer::RenderEvent for #ident #generics {
            const EVENT_NAME: &'static str = "";

            fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                match self {
                    #(
                        Self::#data(v) => v.render_event_data(renderer),
                    )*
                }
            }

            fn event_info(&self) -> jinja_renderer::EventInfo {
                match self {
                    #(
                        Self::#data(v) => v.event_info(),
                    )*
                }
            }
        }
    }
}

fn check_id_exists(data: &darling::ast::Data<EnumData, FieldData>, id_field: &str) -> bool {
    let fields = match data {
        darling::ast::Data::Struct(v) => &v.fields,
        _ => return false,
    };

    for field in fields {
        if let Some(ref ident) = field.ident {
            if ident == id_field {
                return true;
            }
        }
    }
    false
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
    fn event_without_template_and_id_should_work() {
        let options = generate_event_input("foo", false, false);
        let expected = generate_event_expected(false, false);
        let actual = generate_event_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn event_with_template_and_id_should_work() {
        let options = generate_event_input("foo", true, true);
        let expected = generate_event_expected(true, true);
        let actual = generate_event_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn event_with_template_and_without_id_should_work() {
        let options = generate_event_input("foo", true, false);
        let expected = generate_event_expected(true, false);
        let actual = generate_event_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn event_without_template_and_with_id_should_work() {
        let options = generate_event_input("foo", false, true);
        let expected = generate_event_expected(false, true);
        let actual = generate_event_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn enum_event_should_work() {
        let input = parse_quote! {

            #[derive(Debug, Event)]
            pub(crate) enum EnumEvents<'a> {
                Foo2(Foo1<'a>),
                Bar2(Bar1<'a>),
                Baz2(Baz1<'a>),
            }
        };
        let options = EventOptions::from_derive_input(&input).unwrap();
        let expected = quote! {
            impl<'a> jinja_renderer::RenderEvent for EnumEvents<'a> {
                const EVENT_NAME: &'static str = "";

                fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                    match self {
                        Self::Foo2(v) => v.render_event_data(renderer),
                        Self::Bar2(v) => v.render_event_data(renderer),
                        Self::Baz2(v) => v.render_event_data(renderer),
                    }
                }

                fn event_info(&self) -> jinja_renderer::EventInfo {
                    match self {
                        Self::Foo2(v) => v.event_info(),
                        Self::Bar2(v) => v.event_info(),
                        Self::Baz2(v) => v.event_info(),
                    }
                }
            }
        };
        let actual = generate_event_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    fn generate_event_input(name: &str, with_template: bool, with_id: bool) -> EventOptions {
        let top_line = if with_template {
            quote! {
              #[derive(Debug, Serialize, Template, Event)]
              #[template(name = #name)]
            }
        } else {
            quote! {
              #[derive(Debug, Serialize, Event)]
            }
        };
        let (event_line, id_line) = if with_id {
            (
                quote! { #[event(name = "foo", receivers = "#bar", target = "dynamic", swap = "innerHTML", id_prefix = "#my-")] },
                quote! { id: &'a str },
            )
        } else {
            (
                quote! { #[event(name = "foo", receivers = "#bar", target = "#baz", swap = "innerHTML")] },
                quote! {},
            )
        };
        let input = parse_quote! {
            #top_line
            #event_line
            struct Foo<'a> {
                bar: &'a str,
                #id_line
            }
        };

        EventOptions::from_derive_input(&input).unwrap()
    }

    fn generate_event_expected(with_template: bool, with_id: bool) -> TokenStream {
        let target_info = if with_id {
            quote! {
              format!("{}{}", "#my-", self.id).into()
            }
        } else {
            quote! {
              "#baz".into()
            }
        };

        let event_info_with_data = quote! {
            serde_json::to_string(&serde_json::json!({"info": self.event_info(), "data": self})).expect("even info should be a valid json")
        };
        let render_data = if with_template {
            quote! {
                let mut ret = #event_info_with_data;
                let data = self.render(renderer)?;
                ret.push_str("\n");
                ret.push_str(&data);
                Ok(ret)
            }
        } else {
            quote! {
                Ok(#event_info_with_data)
            }
        };

        quote! {
            impl<'a> jinja_renderer::RenderEvent for Foo<'a> {
                const EVENT_NAME: &'static str = "foo";

                fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                    #render_data
                }

                fn event_info(&self) -> jinja_renderer::EventInfo {
                  jinja_renderer::EventInfo {
                    name: "foo",
                    receivers: &["#bar"],
                    target: #target_info,
                    swap: "innerHTML",
                    id_field: "id",
                  }

                }
            }
        }
    }
}
