use free_icons::{FontAwesome, IconAttrs};
use std::borrow::Cow;
type Result<T, E = minijinja::Error> = std::result::Result<T, E>;

pub fn icon(
    s: Cow<'_, str>,
    class: Option<Cow<'_, str>>,
    fill: Option<Cow<'_, str>>,
) -> Result<String> {
    let class = class.as_ref().map(|s| s.as_ref()).unwrap_or_default();
    let fill = fill.as_ref().map(|s| s.as_ref()).unwrap_or("currentColor");
    let attrs = IconAttrs::default().class(class).fill(fill);
    Ok(free_icons::font_awesome(s.as_ref(), FontAwesome::Solid, attrs).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_should_work() {
        insta::assert_snapshot!(icon("github".into(), Some("test".into()), None).unwrap());
    }
}
