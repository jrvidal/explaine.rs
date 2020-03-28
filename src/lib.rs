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
pub struct SessionResult(syn::Result<Session>);

#[wasm_bindgen]
impl SessionResult {
    #[wasm_bindgen]
    pub fn session(self) -> Option<Session> {
        self.0.ok()
    }

    #[wasm_bindgen]
    pub fn error_location(&self) -> Option<Box<[u32]>> {
        self.0.as_ref().err().map(|err| {
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
        self.0.as_ref().err().map(|err| format!("{}", err)).into()
    }
}

#[wasm_bindgen]
pub struct Session {
    code: RustCode,
    positions: IntoIter<(usize, usize)>,
}

enum RustCode {
    File(syn::File),
    Block(syn::Block),
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen]
    pub fn new(source: &str) -> SessionResult {
        utils::set_panic_hook();
        let result = syn::parse_file(source)
            .map(RustCode::File)
            // .or_else(|_| syn::parse_str(&format!("{{\n{}}}", source)).map(RustCode::Block))
            .map(|code| {
                let positions = source
                    .lines()
                    .enumerate()
                    .flat_map(|(i, line)| line.chars().enumerate().map(move |(j, _)| (i + 1, j)))
                    .collect::<Vec<_>>()
                    .into_iter();

                Session { code, positions }
            });
        SessionResult(result)
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
                    if within_spans(loc, last_position.0, last_position.1) {
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
        let offset = self.offset() as usize;

        let location = LineColumn {
            line: line + offset,
            column: ch,
        };
        let visitor = IntersectionVisitor::new(location);
        let result = visitor.visit(&self.code);
        if let HelpItem::Unknown = result.help {
            None
        } else {
            Some(Explanation {
                item: result.help,
                start_line: result.item_location.0.line - offset,
                start_column: result.item_location.0.column,
                end_line: result.item_location.1.line - offset,
                end_column: result.item_location.1.column,
            })
        }
    }

    fn offset(&self) -> u32 {
        if let RustCode::Block(..) = self.code {
            1
        } else {
            0
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

fn within_spans(loc: LineColumn, start: LineColumn, end: LineColumn) -> bool {
    (start.line < loc.line || (start.line == loc.line && start.column <= loc.column))
        && (loc.line < end.line || (loc.line == end.line && loc.column <= end.column))
}
