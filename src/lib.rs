use analyzer::{HelpItem, IntersectionVisitor};
use proc_macro2::{token_stream::IntoIter as TreeIter, LineColumn, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::vec::IntoIter;

mod utils;

use wasm_bindgen::prelude::*;

struct TokenIterator {
    elements: Vec<TokenIteratorElement>,
}

enum TokenIteratorElement {
    Span(Span),
    Tree(TreeIter),
}

impl Iterator for TokenIterator {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.elements.pop() {
            let mut tree_iter = match element {
                TokenIteratorElement::Tree(iter) => iter,
                TokenIteratorElement::Span(span) => {
                    return Some(span);
                }
            };

            let tree = if let Some(tree) = tree_iter.next() {
                self.elements.push(TokenIteratorElement::Tree(tree_iter));
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

            self.elements.push(TokenIteratorElement::Span(span_close));
            self.elements
                .push(TokenIteratorElement::Tree(group.stream().into_iter()));
            return Some(span_open);
        }

        return None;
    }
}

impl From<TokenStream> for TokenIterator {
    fn from(stream: TokenStream) -> Self {
        TokenIterator {
            elements: vec![TokenIteratorElement::Tree(stream.into_iter())],
        }
    }
}

#[cfg(feature = "dev")]
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
    file: syn::File,
    tokens: TokenIterator,
    top_level: IntoIter<usize>,
    element: usize,
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen]
    pub fn new(source: String) -> SessionResult {
        utils::set_panic_hook();
        let parse_result = syn::parse_file(&source);

        let result = match parse_result {
            Ok(file) => {
                let tokens = TokenStream::new().into();
                let mut top_level_elements = vec![];

                top_level_elements.extend(file.attrs.iter().enumerate().map(|(i, _)| i));
                top_level_elements.extend(
                    file.items
                        .iter()
                        .enumerate()
                        .map(|(i, _)| i + file.attrs.len()),
                );

                Ok(Session {
                    file,
                    tokens,
                    top_level: top_level_elements.into_iter(),
                    element: 0,
                })
            }
            Err(err) => {
                let block_result = syn::parse_str::<syn::Block>(&format!("{{{}}}", &source));
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

        loop {
            let span = match self.tokens.next() {
                Some(span) => span,
                None => match self.top_level.next() {
                    Some(el) => {
                        self.element = el;
                        self.tokens = match el {
                            _ if el < self.file.attrs.len() => {
                                self.file.attrs[el].clone().to_token_stream().into()
                            }
                            _ => self.file.items[el - self.file.attrs.len()]
                                .clone()
                                .to_token_stream()
                                .into(),
                        };
                        continue;
                    }
                    None => {
                        self.top_level = vec![].into_iter();
                        self.tokens = TokenIterator { elements: vec![] };
                        break;
                    }
                },
            };

            let location = span.start();

            let explanation = if let Some(explanation) = {
                let visitor = IntersectionVisitor::new(
                    location,
                    #[cfg(feature = "dev")]
                    log,
                );
                let result = visitor.visit_element(&self.file, self.element);
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
    pub fn explain(&self, line: usize, ch: usize) -> Option<Explanation> {
        let location = LineColumn { line, column: ch };
        let visitor = IntersectionVisitor::new(
            location,
            #[cfg(feature = "dev")]
            log,
        );
        let result = visitor.visit(&self.file);
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

    pub fn std(&self) -> JsValue {
        self.item.std().into()
    }
}
