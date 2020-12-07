use analyzer::{Analyzer, ExplorationIterator, ExplorationState, HelpItem, IrVisitor};
use proc_macro2::{
    token_stream::IntoIter as TokenStreamIter, LineColumn, Span, TokenStream, TokenTree,
};
use quote::ToTokens;

mod utils;

use wasm_bindgen::prelude::*;

struct SpanIterator {
    elements: Vec<SpanIteratorElement>,
}

enum SpanIteratorElement {
    Span(Span),
    Tree(TokenStreamIter),
}

impl Iterator for SpanIterator {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.elements.pop() {
            let mut tree_iter = match element {
                SpanIteratorElement::Tree(iter) => iter,
                SpanIteratorElement::Span(span) => {
                    return Some(span);
                }
            };

            let tree = if let Some(tree) = tree_iter.next() {
                self.elements.push(SpanIteratorElement::Tree(tree_iter));
                tree
            } else {
                continue;
            };

            let group = match tree {
                TokenTree::Ident(ident) => {
                    return Some(ident.span());
                }
                TokenTree::Punct(punct) => {
                    return Some(punct.span());
                }
                TokenTree::Literal(lit) => {
                    return Some(lit.span());
                }
                TokenTree::Group(group) => group,
            };

            let span_open = group.span_open();
            let span_close = group.span_close();

            self.elements.push(SpanIteratorElement::Span(span_close));
            self.elements
                .push(SpanIteratorElement::Tree(group.stream().into_iter()));
            return Some(span_open);
        }

        return None;
    }
}

impl From<TokenStream> for SpanIterator {
    fn from(stream: TokenStream) -> Self {
        SpanIterator {
            elements: vec![SpanIteratorElement::Tree(stream.into_iter())],
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "logWasm")]
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
            .map(|(err, _)| err.to_string())
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
    analyzer: Analyzer,
    tokens: SpanIterator,
    state: ExplorationState,
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen]
    pub fn new(source: String) -> SessionResult {
        utils::set_panic_hook();
        #[cfg(feature = "dev")]
        {
            use std::sync::Once;

            static START: Once = Once::new();

            START.call_once(|| {
                console_log::init().unwrap();
            });
        }
        let parse_result = syn::parse_file(&source);

        let result = match parse_result {
            Ok(file) => {
                let token_stream = file.to_token_stream();
                let analyzer = IrVisitor::new(file, source).visit();

                Ok(Session {
                    analyzer,
                    tokens: token_stream.into(),
                    state: ExplorationState::default(),
                })
            }
            Err(err) => {
                let wrapped_code = "{".to_string() + &source + "}";
                let block_result = syn::parse_str::<syn::Block>(&wrapped_code);
                Err((err, block_result.is_ok()))
            }
        };

        SessionResult { result }
    }

    #[wasm_bindgen]
    pub fn explore(&mut self, dest: &mut [usize]) -> usize {
        let max = dest.len() / 4;
        let mut count = 0;
        let mut idx = 0;

        let source = (&mut self.tokens).map(|span| span.start().into());

        let mut exploration_iterator = ExplorationIterator {
            analyzer: &self.analyzer,
            state: &mut self.state,
            source,
        };

        loop {
            let explanation = if let Some(explanation) = {
                let result = if let Some(result) = exploration_iterator.next() {
                    result
                } else {
                    break;
                };

                result.map(|result| Explanation {
                    item: result.help,
                    start_line: result.start.line,
                    start_column: result.start.column,
                    end_line: result.end.line,
                    end_column: result.end.column,
                })
            } {
                explanation
            } else {
                continue;
            };

            #[cfg(feature = "dev")]
            {
                log(&format!("{:?}", explanation));
            }

            dest[idx] = explanation.start_line;
            dest[idx + 1] = explanation.start_column;
            dest[idx + 2] = explanation.end_line;
            dest[idx + 3] = explanation.end_column;

            idx += 4;
            count += 1;

            if count == max {
                break;
            }
        }

        count
    }

    #[wasm_bindgen]
    pub fn explain(&self, line: usize, column: usize) -> Option<Explanation> {
        let location = LineColumn { line, column };

        self.analyzer
            .analyze(location.into())
            .map(|result| Explanation {
                item: result.help,
                start_line: result.start.line,
                start_column: result.start.column,
                end_line: result.end.line,
                end_column: result.end.column,
            })
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

    pub fn extra_info(&self) -> Box<[JsValue]> {
        [
            (self.item.keyword(), "keyword"),
            (self.item.std(), "std"),
            (self.item.book(), "book"),
        ]
        .iter()
        .filter_map(|(entry, kind)| entry.map(|e| (e, *kind)))
        .flat_map(|(entry, kind)| std::iter::once(entry).chain(std::iter::once(kind)))
        .map(JsValue::from)
        .collect::<Box<[_]>>()
    }
}
