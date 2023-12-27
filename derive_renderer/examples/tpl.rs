use derive_jinja_renderer::Template;
use jinja_renderer::{OwnedTemplate, Renderer};
use serde::Serialize;

#[derive(Debug, Serialize, Template)]
#[template(name = "foo.html.j2")]
struct Foo<'a> {
    bar: &'a str,
}

fn main() {
    let data = Foo { bar: "baz" };

    let templates = vec![OwnedTemplate::new(
        "foo.html.j2",
        "<html>\n  <body>\n    {{ bar }}\n  </body>\n</html>",
    )];

    let mut renderer = Renderer::default();
    renderer.add_templates(templates.into_iter()).unwrap();

    let rendered = renderer.render(&data).unwrap();

    println!("{:?}", rendered);
}
