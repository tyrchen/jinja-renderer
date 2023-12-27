use crate::TemplateOptions;
// only proc_macro2::TokenStream is testable
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate(options: TemplateOptions) -> TokenStream {
    let TemplateOptions {
        ident,
        generics,
        name,
        mime,
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
            fn template_name(&self) -> &'static str {
                #name
            }

            #mime_code
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
    fn macro_should_work() {
        let options = generate_input("foo.html.j2", None);
        let expected = generate_expected("foo.html.j2", "text/html; charset=utf-8");
        let actual = generate(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn macro_default_mime_should_work() {
        let options = generate_input("foo.js.j2", None);
        let expected = generate_expected("foo.js.j2", "text/plain; charset=utf-8");
        let actual = generate(options).to_string();
        assert_eq!(actual, expected.to_string());
    }
    #[test]
    fn macro_with_mime_should_work() {
        let options = generate_input("foo.json.j2", Some("application/json"));
        let expected = generate_expected("foo.json.j2", "application/json");
        let actual = generate(options).to_string();
        assert_eq!(actual, expected.to_string());
    }

    fn generate_input(name: &str, mime: Option<&str>) -> TemplateOptions {
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

    fn generate_expected(name: &str, mime: &str) -> TokenStream {
        quote! {
            impl<'a> jinja_renderer::RenderContext for Foo<'a> {
                fn template_name(&self) -> &'static str {
                    #name
                }

                const MIME_TYPE: &'static str = #mime;
            }
        }
    }
}
