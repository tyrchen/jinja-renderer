mod frameworks;

#[cfg(feature = "minify")]
use minify_html::{minify, Cfg};
use minijinja::Environment;
use serde::Serialize;
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

pub mod filters;

pub use minijinja::Error;

#[cfg(feature = "derive")]
pub use derive_jinja_renderer::*;

#[cfg(feature = "minify")]
const CFG: Cfg = Cfg {
    do_not_minify_doctype: true,
    ensure_spec_compliant_unquoted_attribute_values: true,
    keep_closing_tags: true,
    keep_html_and_head_opening_tags: true,
    keep_spaces_between_attributes: true,
    keep_input_type_text_attr: true,
    preserve_brace_template_syntax: true,
    minify_css: false,
    minify_js: false,
    keep_comments: false,
    keep_ssi_comments: false,
    preserve_chevron_percent_template_syntax: false,
    remove_bangs: false,
    remove_processing_instructions: false,
};

pub trait RenderContext: Serialize {
    /// The name of the template to render
    const TEMPLATE_NAME: &'static str;
    /// The MIME type (Content-Type) of the data that gets rendered by this Template
    const MIME_TYPE: &'static str;
    /// render the context data
    fn render(&self, renderer: &Renderer) -> Result<String, Error>;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInfo {
    pub name: &'static str,
    pub receivers: &'static [&'static str],
    pub target: Cow<'static, str>,
    pub swap: &'static str,
    pub id_field: &'static str,
}

pub trait RenderEvent {
    /// the event name
    const EVENT_NAME: &'static str;
    /// render the event data for SSE with the format as `encoded_json\nencoded_html`
    fn render_event_data(&self, renderer: &Renderer) -> Result<String, Error>;
    // event id
    fn event_info(&self) -> EventInfo;
}

pub struct OwnedTemplate {
    pub name: Cow<'static, str>,
    pub data: Cow<'static, str>,
}

#[derive(Debug)]
pub struct Renderer(Environment<'static>);

impl Deref for Renderer {
    type Target = Environment<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(Environment::new())
    }
}

impl OwnedTemplate {
    pub fn new(name: impl Into<Cow<'static, str>>, data: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
        }
    }
}

impl Renderer {
    pub fn new(env: Environment<'static>) -> Self {
        Self(env)
    }

    pub fn add_templates(
        &mut self,
        templates: impl Iterator<Item = OwnedTemplate>,
    ) -> Result<(), Error> {
        for tpl in templates {
            self.add_template_owned(tpl.name, tpl.data)?;
        }
        Ok(())
    }

    pub fn render_template<T: Serialize>(&self, name: &str, context: &T) -> Result<String, Error> {
        let tpl = self.0.get_template(name)?;
        let mime = if name.ends_with("html.j2") {
            "text/html; charset=utf-8"
        } else if name.ends_with("json.j2") {
            "application/json; charset=utf-8"
        } else {
            "text/plain; charset=utf-8"
        };
        self.render_minified(tpl, mime, context)
    }

    fn render_minified(
        &self,
        tpl: minijinja::Template<'_, '_>,
        #[allow(unused_variables)] mime: &str,
        context: &impl Serialize,
    ) -> Result<String, Error> {
        #[cfg(feature = "minify")]
        {
            let ret = tpl.render(context)?;
            if mime.starts_with("text/html") {
                let minified = minify(ret.as_bytes(), &CFG);
                Ok(unsafe { String::from_utf8_unchecked(minified) })
            } else {
                Ok(ret)
            }
        }
        #[cfg(not(feature = "minify"))]
        {
            tpl.render(context)
        }
    }
}
