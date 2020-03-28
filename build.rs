use comrak::{markdown_to_html, ComrakOptions};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::{Cursor, Write};

const BOOK: &str = "https://doc.rust-lang.org/book/";

fn main() {
    println!("cargo:rerun-if-changed=explainers.toml");

    let explanations_toml = std::fs::read_to_string("explainers.toml").unwrap();
    let explanations: BTreeMap<String, Explanation> = toml::from_str(&explanations_toml).unwrap();

    let source = "fn help_to_message(item: HelpItem) -> (&'static str, &'static str, Option<&'static str>, Option<&'static str>) {
    match item {
      HelpItem::Unknown => (\"\", \"\", None, None),
  "
    .to_string()
    .into_bytes();

    let mut cursor = Cursor::new(source);
    cursor.set_position(cursor.get_ref().len() as u64);

    for (name, explanation) in explanations {
        let pattern = explanation.pattern.unwrap_or_else(|| "".to_string());
        let variant = explanation.variant.as_ref().unwrap_or(&name);
        let title = explanation.title.as_ref().unwrap_or(&name);

        let rendered_title = markdown_to_html(title, &ComrakOptions::default());
        let stripped_title = rendered_title
            .trim_start_matches("<p>")
            .trim_end_matches("</p>\n");

        let book = explanation
            .book
            .map(|book| format!("Some({:?})", format!("{}{}", BOOK, book)))
            .unwrap_or("None".to_string());

        let keyword = explanation
            .keyword
            .map(|keyword| {
                format!(
                    "Some({:?})",
                    format!("https://doc.rust-lang.org/std/keyword.{}.html", keyword)
                )
            })
            .unwrap_or("None".to_string());

        std::writeln!(
            &mut cursor,
            "  HelpItem::{} {} => ({:?}, {:?}, {}, {}),",
            variant,
            pattern,
            &markdown_to_html(&explanation.info, &ComrakOptions::default()),
            stripped_title,
            book,
            keyword
        )
        .unwrap();
    }

    std::writeln!(&mut cursor, "  }}\n}}").unwrap();

    std::fs::write(
        std::env::var("OUT_DIR").unwrap() + "/help.rs",
        &cursor.into_inner(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct Explanation {
    info: String,
    pattern: Option<String>,
    variant: Option<String>,
    title: Option<String>,
    book: Option<String>,
    keyword: Option<String>,
}
