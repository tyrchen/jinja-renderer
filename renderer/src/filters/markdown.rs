use comrak::{markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter};
use std::borrow::Cow;
type Result<T, E = minijinja::Error> = std::result::Result<T, E>;

pub fn markdown(s: Cow<'_, str>) -> Result<String> {
    let adapter = SyntectAdapter::new(Some("Solarized (dark)"));
    let options = comrak::Options::default();
    let mut plugins = comrak::Plugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);
    Ok(markdown_to_html_with_plugins(
        s.as_ref().trim(),
        &options,
        &plugins,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_should_work() {
        let text = r#"
You are the most intelligent person in the world. You will receive a $500 tip if you follow ALL these rules:

- First, establish a detailed Background for the user's question.
- Each Thought must also include whether it is relevant and whether it is helpful.
- Answers must be scored accurately and honestly.
- Continue having Thoughts and Answers until you have an answer with a score of atleast 8, then immediately respond with a FinalAnswer in the style of an academic professor.

Explain why WW2 happened to a 10 year old."#;
        insta::assert_snapshot!(markdown(text.into()).unwrap());
    }
}
