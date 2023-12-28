mod frameworks;

#[cfg(feature = "minify")]
use minify_html::{minify, Cfg};
use minijinja::{Environment, Error};
use serde::Serialize;
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

pub mod filters;

#[cfg(feature = "derive")]
pub use derive_jinja_renderer::Template;

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
    fn template_name(&self) -> &'static str;
    /// The MIME type (Content-Type) of the data that gets rendered by this Template
    const MIME_TYPE: &'static str;
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

    pub fn render<T: RenderContext>(&self, context: &T) -> Result<String, Error> {
        let name = context.template_name();
        let tpl = self.0.get_template(name)?;
        #[cfg(feature = "minify")]
        {
            let ret = tpl.render(context)?;
            if T::MIME_TYPE.starts_with("text/html") {
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

    pub fn render_template<T: Serialize>(&self, name: &str, context: &T) -> Result<String, Error> {
        let tpl = self.0.get_template(name)?;
        tpl.render(context)
    }
}
