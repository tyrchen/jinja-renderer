use crate::{AllEventsOptions, EventOptions, FieldData, TemplateOptions};
// only proc_macro2::TokenStream is testable
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::json;

pub(crate) fn generate_render_context_trait(options: TemplateOptions) -> TokenStream {
    let TemplateOptions {
        ident,
        generics,
        name,
        mime,
        ..
    } = options;

    let mime_code = if let Some(mime) = mime {
        quote! { const MIME_TYPE: &'static str = #mime; }
    } else if name.ends_with("html.j2") {
        quote! { const MIME_TYPE: &'static str = "text/html; charset=utf-8"; }
    } else if name.ends_with("json.j2") {
        quote! { const MIME_TYPE: &'static str = "application/json; charset=utf-8"; }
    } else {
        quote! { const MIME_TYPE: &'static str = "text/plain; charset=utf-8"; }
    };
    quote! {
        impl #generics jinja_renderer::RenderContext for #ident #generics {
            const TEMPLATE_NAME: &'static str = #name;
            #mime_code

            fn render_context(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                renderer.render_template(#name, &self)
            }
        }
    }
}

pub(crate) fn generate_event_trait(options: EventOptions) -> TokenStream {
    let EventOptions {
        ident,
        generics,
        name,
        receivers,
        target,
        swap,
        data,
        attrs,
    } = options;

    let receivers = receivers
        .trim()
        .split(' ')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    if name.is_empty() || receivers.is_empty() || target.is_empty() {
        panic!("event attrs must not be empty");
    }

    // TODO: use regex to do more tests
    for receiver in &receivers {
        if !receiver.starts_with('#') {
            panic!("receivers must start with #, and separated by space");
        }
    }

    let event_info_code = if target == "dynamic" {
        if !check_id_exists(&data) {
            panic!("target is dynamic, but id field not found");
        } else {
            quote! {
              fn event_info(&self) -> String {
                serde_json::to_string(&serde_json::json!({
                    "receivers": &[#(#receivers),*],
                    "target":  format!("#{}", self.id),
                    "swap": #swap,
                }))
                .expect("even info should be a valid json")
              }
            }
        }
    } else {
        let json = serde_json::to_string(&json!({
            "receivers": receivers,
            "target": target,
            "swap": swap,
        }))
        .expect("even info should be a valid json");
        quote! {
          fn event_info(&self) -> String {
            #json.to_string()
          }
        }
    };

    let render_event_code = if !attrs.is_empty() {
        quote! {
            use jinja_renderer::RenderContext;
            let mut ret = self.event_info();
            let data = self.render_context(renderer)?;
            ret.push_str("\n");
            ret.push_str(&data);
            Ok(ret)
        }
    } else {
        quote! {
          Ok(self.event_info())
        }
    };

    quote! {
        impl #generics jinja_renderer::RenderEvent for #ident #generics {
            const RECEIVERS: &'static [&'static str] = &[#(#receivers),*];
            const EVENT_NAME: &'static str = #name;
            const EVENT_TARGET: &'static str = #target;
            const EVENT_SWAP: &'static str = #swap;

            fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                #render_event_code
            }

            #event_info_code
        }


    }
}

pub(crate) fn generate_all_events_fn(options: AllEventsOptions) -> TokenStream {
    let ident = options.ident;
    let generics = options.generics;
    let vis = options.vis;
    let data = match options.data {
        darling::ast::Data::Enum(v) => v
            .into_iter()
            .map(|v| match v.fields.style {
                darling::ast::Style::Tuple => {
                    if v.fields.fields.len() != 1 {
                        panic!("AllEvents only support tuple struct with one field");
                    }
                    let ty = &v.fields.fields[0].ty;
                    let names = match ty {
                        syn::Type::Path(p) => {
                            let path = &p.path;
                            path.segments.iter().map(|s| &s.ident).collect::<Vec<_>>()
                        }
                        _ => panic!("AllEvents only support tuple struct with one field"),
                    };
                    quote! { #(#names)::*::EVENT_NAME }
                }
                _ => panic!("AllEvents only support tuple struct"),
            })
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

fn check_id_exists(data: &darling::ast::Data<darling::util::Ignored, FieldData>) -> bool {
    let fields = match data {
        darling::ast::Data::Struct(v) => &v.fields,
        _ => return false,
    };

    for field in fields {
        if let Some(ref ident) = field.ident {
            if ident == "id" {
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

    #[test]
    fn macro_should_work() {
        let options = generate_template_input("foo.html.j2", None);
        let expected = generate_template_expected("foo.html.j2", "text/html; charset=utf-8");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn macro_default_mime_should_work() {
        let options = generate_template_input("foo.js.j2", None);
        let expected = generate_template_expected("foo.js.j2", "text/plain; charset=utf-8");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }
    #[test]
    fn macro_with_mime_should_work() {
        let options = generate_template_input("foo.json.j2", Some("application/json"));
        let expected = generate_template_expected("foo.json.j2", "application/json");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
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

    // private functions
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
                quote! { #[event(name = "foo", receivers = "#bar", target = "dynamic", swap = "innerHTML")] },
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
        let (event_target, event_info) = if with_id {
            (
                quote! { "dynamic" },
                quote! {
                    fn event_info(&self) -> String {
                        serde_json::to_string(&serde_json::json!({
                            "receivers": &["#bar"],
                            "target":  format!("#{}", self.id),
                            "swap": "innerHTML",
                        }))
                        .expect("even info should be a valid json")
                    }
                },
            )
        } else {
            let json = serde_json::to_string(&json!({
                "receivers": ["#bar"],
                "target": "#baz",
                "swap": "innerHTML",
            }))
            .unwrap();
            (
                quote! { "#baz" },
                quote! {
                    fn event_info(&self) -> String {
                      #json.to_string ()
                    }
                },
            )
        };

        let render_data = if with_template {
            quote! {
                use jinja_renderer::RenderContext;
                let mut ret = self.event_info();
                let data = self.render_context(renderer)?;
                ret.push_str("\n");
                ret.push_str(&data);
                Ok(ret)
            }
        } else {
            quote! {
                Ok(self.event_info())
            }
        };

        quote! {
            impl<'a> jinja_renderer::RenderEvent for Foo<'a> {
                const RECEIVERS: &'static [&'static str] = &["#bar"];
                const EVENT_NAME: &'static str = "foo";
                const EVENT_TARGET: &'static str = #event_target;
                const EVENT_SWAP: &'static str = "innerHTML";

                fn render_event_data(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                    #render_data
                }

                #event_info
            }
        }
    }

    fn generate_template_input(name: &str, mime: Option<&str>) -> TemplateOptions {
        let line = if let Some(mime) = mime {
            quote! { #[template(name = #name, mime = #mime)] }
        } else {
            quote! { #[template(name = #name)] }
        };
        let input = parse_quote! {
            #[derive(Debug, Serialize, Template)]
            #line
            struct Foo<'a> {
                bar: &'a str,
            }
        };

        TemplateOptions::from_derive_input(&input).unwrap()
    }

    fn generate_template_expected(name: &str, mime: &str) -> TokenStream {
        quote! {
            impl<'a> jinja_renderer::RenderContext for Foo<'a> {
                const TEMPLATE_NAME: &'static str = #name;
                const MIME_TYPE: &'static str = #mime;
                fn render_context(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                    renderer.render_template(#name, &self)
                }
            }
        }
    }
}
