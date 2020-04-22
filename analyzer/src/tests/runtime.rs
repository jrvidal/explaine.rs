use crate::{ir::Location, HelpItem};
use serde_yaml;
use std::borrow::Cow;
use std::{rc::Rc, str::FromStr};

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
    span: Option<(Location, Location)>,
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
                Location {
                    line: start[0],
                    column: start[1],
                },
                Location {
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

    if item.is_some() != span.is_some() {
        panic!("Item and location must be specified together");
    }

    RunData {
        expected_item: item.unwrap_or(HelpItem::Unknown),
        span,
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

    assert_eq!(cursors.len(), 1, "Only one cursor per spec");
    let (line, (column, _)) = cursors.pop().unwrap();

    source_lines[line]
        .to_mut()
        .replace_range(column..(column + 3), "");

    if !run_data.naked {
        source_lines.insert(0, "fn __main() {".into());
        source_lines.push("}".into());
    }

    let test_source = source_lines.join("\n");
    let line_info = test_source.lines().map(|l| l.len()).collect();
    let file = syn::parse_file(&test_source).expect("invalid source");
    let ir_visitor = crate::ir::IrVisitor::new(Rc::new(file), line_info);

    let analyzer = ir_visitor.visit();

    let result = if let Some(result) = analyzer.analyze(Location {
        line: line + 1 + offset,
        column,
    }) {
        result
    } else {
        panic!("Should find help (case #{})", case);
    };

    assert_eq!(run_data.expected_item, result.help, "Case #{}", case);
    let adjusted = (
        Location {
            line: result.start.line - offset,
            ..result.start
        },
        Location {
            line: result.end.line - offset,
            ..result.end
        },
    );
    if let Some((start, end)) = run_data.span {
        assert_eq!((start, end), adjusted, "Case {}", case);
    }
}
