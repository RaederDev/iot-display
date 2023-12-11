use std::collections::HashMap;

use anyhow::{bail, Error, Ok};
use dioxus::prelude::*;

use super::widget::Widget;

#[derive(PartialEq, Props)]
pub struct TextWidget {
    content: String,
}

impl Widget for TextWidget {
    fn render<'a>(&self, cx: &'a ScopeState) -> Element<'a> {
        let mut count = use_state(cx, || 0);

        cx.render(rsx!(
            h1 { "High-Five counter: {count}" }
            button {
                onclick: move |_| {
                    count += 1
                },
                "Up high!"
            }
            button {
                onclick: move |_| {
                    count -= 1
                },
                "Down low!"
            }
            div {
                "Content: {self.content}"
            }
        ))
    }

    fn build(config: &Option<HashMap<String, String>>) -> Result<Box<dyn Widget>, Error> {
        if config.is_none() {
            bail!("Missing configuration for TextWidget");
        }
        match config.as_ref().unwrap().get("content") {
            Some(content) => Ok(Box::from(TextWidget {
                content: content.to_string(),
            })),
            None => bail!("Missing config key content for TextWidget"),
        }
    }
}
