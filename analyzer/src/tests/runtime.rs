use crate::{HelpItem, IntersectionVisitor};
use proc_macro2::LineColumn;
use serde_yaml;
use std::borrow::Cow;
use std::str::FromStr;

pub fn test_example(source: &str) {
    let lines: Vec<_> = source.lines().collect();
    let blocks = lines.iter().fold(vec![vec![]], |mut acc, line| {
        if line.starts_with("---") {
            acc.push(vec![]);
            acc
        } else {
            acc.last_mut().unwrap().push(*line);
            acc
        }
    });

    assert!(blocks.len() % 2 == 0, "data + source");

    for case in 0..(blocks.len() / 2) {
        let data_lines = &blocks[2 * case];
        let code_lines = &blocks[2 * case + 1];
        let run_data = parse_run_data(&data_lines[..]);
        run_case(code_lines, run_data, case);
    }
}

struct RunData {
    naked: bool,
    expected_item: HelpItem,
    start: LineColumn,
    end: LineColumn,
}

fn parse_run_data(lines: &[&str]) -> RunData {
    let mut item: Option<HelpItem> = None;
    let mut span = None;
    let mut naked = false;

    for line in lines {
        if line.starts_with("span:") {
            let span_components: Vec<_> = line[("span:".len())..].trim().split("=>").collect();
            assert_eq!(span_components.len(), 2);
            let start: [usize; 2] =
                serde_yaml::from_str(span_components[0].trim()).expect("span format");
            let end: [usize; 2] =
                serde_yaml::from_str(span_components[1].trim()).expect("span format");
            span = Some((
                LineColumn {
                    line: start[0],
                    column: start[1],
                },
                LineColumn {
                    line: end[0],
                    column: end[1],
                },
            ));
            continue;
        }
        if line.starts_with("item:") {
            let item_line = &line[("item:".len())..].trim();
            let variant = item_line.split(" ").next().expect("variant first element");
            let data_line = format!("{{{}}}", &item_line[variant.len()..].trim());
            let mut help_data: serde_yaml::Mapping =
                serde_yaml::from_str(&data_line).expect("valid YAML");
            help_data.insert("type".into(), variant.into());
            let help_item: HelpItem =
                serde_yaml::from_value(help_data.into()).expect("item parsing");
            item = Some(help_item);
            continue;
        }
        if line.starts_with("naked:") {
            let naked_line = &line["naked:".len()..].trim();

            naked = bool::from_str(naked_line).expect("naked should be boolean");
            continue;
        }
        panic!("Unknown directive {:?}", line);
    }

    let (start, end) = span.expect("no span found");

    RunData {
        expected_item: item.expect("not item found"),
        start,
        end,
        naked,
    }
}

fn run_case(code: &[&str], run_data: RunData, case: usize) {
    let offset = if run_data.naked { 0 } else { 1 };

    let mut source_lines = code
        .iter()
        .map(|l| Cow::Borrowed::<'_, str>(l))
        .collect::<Vec<_>>();

    let mut cursors = source_lines
        .iter()
        .enumerate()
        .flat_map(|(ln, l)| l.match_indices("<|>").map(move |m| (ln, m)))
        .collect::<Vec<_>>();

    assert_eq!(cursors.len(), 1);
    let (line, (column, _)) = cursors.pop().unwrap();

    source_lines[line]
        .to_mut()
        .replace_range(column..(column + 3), "");

    if !run_data.naked {
        source_lines.insert(0, "fn __main() {".into());
        source_lines.push("}".into());
    }

    let test_source = source_lines.join("\n");
    let file = syn::parse_file(&test_source).expect("invalid source");
    let visitor = IntersectionVisitor::new(LineColumn {
        line: line + 1 + offset,
        column,
    });

    let result = visitor.visit(&file);

    assert_eq!(run_data.expected_item, result.help, "Case #{}", case);
    let adjusted = (
        LineColumn {
            line: result.item_location.0.line - offset,
            ..result.item_location.0
        },
        LineColumn {
            line: result.item_location.1.line - offset,
            ..result.item_location.1
        },
    );
    assert_eq!((run_data.start, run_data.end), adjusted, "Case {}", case);
}
