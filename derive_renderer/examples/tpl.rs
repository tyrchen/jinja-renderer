use derive_jinja_renderer::{AllEvents, Event, Template};
use jinja_renderer::{OwnedTemplate, RenderEvent, Renderer};
use serde::Serialize;

#[derive(Debug, Serialize, Template, Event)]
#[template(name = "foo.html.j2")]
#[event(name = "foo", receivers = "#bar", target = "#baz", swap = "innerHTML")]
struct Foo<'a> {
    bar: &'a str,
}

#[derive(Debug, Serialize, Event)]
#[event(name = "bar", receivers = "#bar", target = "#baz", swap = "innerHTML")]
struct Bar<'a> {
    bar: &'a str,
}

#[derive(Debug, Serialize, Event)]
#[event(
    name = "bar",
    receivers = "#bar",
    target = "dynamic",
    id_prefix = "foo-",
    id_field = "foo_id",
    swap = "innerHTML"
)]
struct Baz<'a> {
    bar: &'a str,
    foo_id: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, AllEvents)]
enum AllEvents<'a> {
    Foo1(Foo<'a>),
    Bar1(Bar<'a>),
    Baz1(Baz<'a>),
}

fn main() {
    let templates = vec![OwnedTemplate::new(
        "foo.html.j2",
        "<html>\n  <body>\n    {{ bar }}\n  </body>\n</html>",
    )];

    let mut renderer = Renderer::default();
    renderer.add_templates(templates.into_iter()).unwrap();

    println!(
        "Generated:\n\n{}\n\n{}\n\n{}",
        Foo { bar: "baz" }.render_event_data(&renderer).unwrap(),
        Bar { bar: "baz" }.render_event_data(&renderer).unwrap(),
        Baz {
            bar: "baz",
            foo_id: "1b282198-a671-11ee-ab45-76ae4616a5de"
        }
        .render_event_data(&renderer)
        .unwrap(),
    );

    println!("all events: {:?}", AllEvents::all_events());
}
