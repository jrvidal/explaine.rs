use proc_macro2::LineColumn;
use std::vec::IntoIter;

use syntax::{HelpItem, IntersectionVisitor};

mod syntax;
mod utils;

use wasm_bindgen::prelude::*;

#[cfg(feature = "dev")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct SessionResult {
    result: Result<Session, (syn::Error, bool)>,
}

#[wasm_bindgen]
impl SessionResult {
    #[wasm_bindgen]
    pub fn session(self) -> Option<Session> {
        self.result.ok()
    }

    #[wasm_bindgen]
    pub fn error_location(&self) -> Option<Box<[u32]>> {
        self.result.as_ref().err().map(|(err, _)| {
            let span = err.span();
            vec![
                span.start().line as u32,
                span.start().column as u32,
                span.end().line as u32,
                span.end().column as u32,
            ]
            .into_boxed_slice()
        })
    }

    #[wasm_bindgen]
    pub fn error_message(&self) -> JsValue {
        self.result
            .as_ref()
            .err()
            .map(|(err, _)| format!("{}", err))
            .into()
    }

    #[wasm_bindgen]
    pub fn is_block(&self) -> bool {
        self.result
            .as_ref()
            .err()
            .map(|&(_, is_block)| is_block)
            .unwrap_or(false)
    }
}

#[wasm_bindgen]
pub struct Session {
    code: syn::File,
    positions: IntoIter<(usize, usize)>,
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen]
    pub fn new(source: &str) -> SessionResult {
        utils::set_panic_hook();
        let result = syn::parse_file(source)
            .map(|code| {
                let positions = source
                    .lines()
                    .enumerate()
                    .flat_map(|(i, line)| line.chars().enumerate().map(move |(j, _)| (i + 1, j)))
                    .collect::<Vec<_>>()
                    .into_iter();

                Session { code, positions }
            })
            .map_err(|err| {
                let block_result = syn::parse_str::<syn::Block>(&format!("{{{}}}", source));
                (err, block_result.is_ok())
            });
        SessionResult { result }
    }

    #[wasm_bindgen]
    pub fn explore(&mut self, dest: &mut [usize], max: usize) -> usize {
        let mut last: Option<(_, _)> = None;
        let mut count = 0;

        for i in (0..dest.len()).step_by(4) {
            let mut found = false;
            while let Some((line, column)) = self.positions.next() {
                if let Some(last_position) = last {
                    let loc = LineColumn { line, column };
                    if within_locations(loc, last_position.0, last_position.1) {
                        continue;
                    } else {
                        last = None;
                    }
                }

                let explanation = if let Some(explanation) = self.explain(line, column) {
                    explanation
                } else {
                    continue;
                };

                #[cfg(feature = "dev")]
                {
                    log(&format!("{:?}", explanation));
                }

                last = Some((
                    LineColumn {
                        line: explanation.start_line,
                        column: explanation.start_column,
                    },
                    LineColumn {
                        line: explanation.end_line,
                        column: explanation.end_column,
                    },
                ));

                dest[i] = explanation.start_line;
                dest[i + 1] = explanation.start_column;
                dest[i + 2] = explanation.end_line;
                dest[i + 3] = explanation.end_column;
                count += 1;
                found = true;
                break;
            }

            if count == max || !found {
                break;
            }
        }

        count
    }

    #[wasm_bindgen]
    pub fn explain(&self, line: usize, ch: usize) -> Option<Explanation> {
        let location = LineColumn {
            line: line,
            column: ch,
        };
        let visitor = IntersectionVisitor::new(location);
        let result = visitor.visit(&self.code);
        if let HelpItem::Unknown = result.help {
            None
        } else {
            Some(Explanation {
                item: result.help,
                start_line: result.item_location.0.line,
                start_column: result.item_location.0.column,
                end_line: result.item_location.1.line,
                end_column: result.item_location.1.column,
            })
        }
    }
}

#[wasm_bindgen]
#[cfg_attr(feature = "dev", derive(Debug))]
pub struct Explanation {
    item: HelpItem,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[wasm_bindgen]
impl Explanation {
    pub fn elaborate(&self) -> JsValue {
        self.item.message().into()
    }

    pub fn title(&self) -> JsValue {
        self.item.title().into()
    }

    pub fn keyword(&self) -> JsValue {
        self.item.keyword().into()
    }

    pub fn book(&self) -> JsValue {
        self.item.book().into()
    }
}

#[cfg(features = "dev")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn within_locations(loc: LineColumn, start: LineColumn, end: LineColumn) -> bool {
    (start.line < loc.line || (start.line == loc.line && start.column <= loc.column))
        && (loc.line < end.line || (loc.line == end.line && loc.column <= end.column))
}
