use crate::TemplateOptions;
// only proc_macro2::TokenStream is testable
use proc_macro2::TokenStream;
use quote::quote;

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

            fn render(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                renderer.render_template(#name, &self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::FromDeriveInput;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn context_should_work() {
        let options = generate_template_input("foo.html.j2", None);
        let expected = generate_template_expected("foo.html.j2", "text/html; charset=utf-8");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn context_default_mime_should_work() {
        let options = generate_template_input("foo.js.j2", None);
        let expected = generate_template_expected("foo.js.j2", "text/plain; charset=utf-8");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn context_with_mime_should_work() {
        let options = generate_template_input("foo.json.j2", Some("application/json"));
        let expected = generate_template_expected("foo.json.j2", "application/json");
        let actual = generate_render_context_trait(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    // private functions

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
                fn render(&self, renderer: &jinja_renderer::Renderer) -> Result<String, jinja_renderer::Error> {
                    renderer.render_template(#name, &self)
                }
            }
        }
    }
}
