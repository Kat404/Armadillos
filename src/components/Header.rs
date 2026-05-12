use maud::{Markup, html};

pub fn header(title: &str) -> Markup {
    html! {
        header {
            h1 {
                (title)
            }
        }
    }
}
