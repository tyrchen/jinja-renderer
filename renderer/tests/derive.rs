use jinja_renderer::{OwnedTemplate, RenderContext, Renderer, Template};
use once_cell::sync::OnceCell;
use serde::Serialize;

static RENDERER: OnceCell<Renderer> = OnceCell::new();

#[derive(Debug, Serialize, Template)]
#[template(name = "foo.html.j2")]
struct Foo<'a> {
    bar: &'a str,
}

#[cfg(feature = "minify")]
#[test]
fn test_minify() {
    let data = Foo { bar: "baz" };
    let renderer = get_render();
    let ret = data.render(renderer).unwrap();

    assert_eq!(ret, "<html><body>baz</body></html>");
}

#[cfg(not(feature = "minify"))]
#[test]
fn test_not_minify() {
    let data = Foo { bar: "baz" };
    let renderer = get_render();
    let ret = data.render(renderer).unwrap();

    assert_eq!(ret, "<html>\n  <body>\n    baz\n  </body>\n</html>");
}

fn get_render() -> &'static Renderer {
    RENDERER.get_or_init(|| {
        let templates = vec![OwnedTemplate::new(
            "foo.html.j2",
            "<html>\n  <body>\n    {{ bar }}\n  </body>\n</html>",
        )];

        let mut renderer = Renderer::default();
        renderer.add_templates(templates.into_iter()).unwrap();

        renderer
    })
}
