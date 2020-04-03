use comrak::{markdown_to_html, ComrakOptions};
use serde::Deserialize;
use std::collections::BTreeMap;

const BOOK: &str = "https://doc.rust-lang.org/book/";

fn main() {
    println!("cargo:rerun-if-changed=explainers.toml");

    let explanations_toml = std::fs::read_to_string("explainers.toml").unwrap();
    let config: Config = toml::from_str(&explanations_toml).unwrap();

    let mut data = vec![];
    let mut init = vec![];

    for (name, explanation) in config.help {
        let rendered_title = markdown_to_html(&explanation.title, &ComrakOptions::default());
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

        let pattern = explanation.pattern.as_ref().map(|s| &s[..]).unwrap_or("..");

        let variant = explanation
            .variant
            .as_ref()
            .map(|s| &s[..])
            .unwrap_or(&name);

        data.push(format!(
            "  HelpItem::{variant} {{{pattern}}} => HelpData {{ template: {template:?}, title: {title:?}, book: {book}, keyword: {keyword} }},\n",
            variant = variant,
            pattern = pattern,
            template = name,
            title = stripped_title,
            book = book,
            keyword = keyword
        ));

        init.push(format!(
            "add_template({:?}, {:?})",
            name,
            &markdown_to_html(&explanation.info, &ComrakOptions::default())
        ));
    }

    let mut source = String::new();

    source.push_str("
        fn help_to_template_data(item: &HelpItem) -> HelpData {
            match item {
                HelpItem::Unknown => HelpData { template: \"\", book: None, keyword: None, title: \"\"},
    ");

    data.into_iter().for_each(|case| {
        source.push_str(&case);
    });

    source.push_str("}\n}\n");

    source.push_str(
        "
        fn init_template() -> tinytemplate::TinyTemplate<'static> {
            let mut template = tinytemplate::TinyTemplate::new();
    ",
    );

    init.into_iter().for_each(|init_call| {
        source.push_str(&format!("template.{}.unwrap();\n", init_call));
    });

    source.push_str("template\n}");

    std::fs::write(std::env::var("OUT_DIR").unwrap() + "/help.rs", &source).unwrap();
}

#[derive(Deserialize)]
struct Config {
    help: BTreeMap<String, Explanation>,
}

#[derive(Deserialize)]
struct Explanation {
    info: String,
    title: String,
    pattern: Option<String>,
    variant: Option<String>,
    book: Option<String>,
    keyword: Option<String>,
}
