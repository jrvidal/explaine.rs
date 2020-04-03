use proc_macro2::{token_stream::IntoIter as TreeIter, LineColumn, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::vec::IntoIter;

use syntax::{HelpItem, IntersectionVisitor};

mod syntax;
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
    tokens: TokenIterator,
    top_level: IntoIter<TopLevelElement>,
    element: TopLevelElement,
}

#[derive(Clone)]
enum TopLevelElement {
    Attr(usize),
    Item(usize),
}

#[wasm_bindgen]
impl Session {
    #[wasm_bindgen]
    pub fn new(source: &str) -> SessionResult {
        utils::set_panic_hook();
        let result = syn::parse_file(source)
            .map(|code| {
                let tokens = TokenStream::new().into();
                let mut top_level_elements = vec![];

                top_level_elements.extend(
                    code.attrs
                        .iter()
                        .enumerate()
                        .map(|(i, _)| TopLevelElement::Attr(i)),
                );
                top_level_elements.extend(
                    code.items
                        .iter()
                        .enumerate()
                        .map(|(i, _)| TopLevelElement::Item(i)),
                );

                Session {
                    code,
                    tokens,
                    top_level: top_level_elements.into_iter(),
                    element: TopLevelElement::Attr(0),
                }
            })
            .map_err(|err| {
                let block_result = syn::parse_str::<syn::Block>(&format!("{{{}}}", source));
                (err, block_result.is_ok())
            });
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
                        self.element = el.clone();
                        self.tokens = match el {
                            TopLevelElement::Attr(i) => {
                                self.code.attrs[i].clone().to_token_stream().into()
                            }
                            TopLevelElement::Item(i) => {
                                self.code.items[i].clone().to_token_stream().into()
                            }
                        };
                        continue;
                    }
                    None => break,
                },
            };

            let location = span.start();

            let explanation = if let Some(explanation) = {
                let visitor = IntersectionVisitor::new(location);
                let result = match self.element {
                    TopLevelElement::Attr(i) => visitor.visit_attr(&self.code, i),
                    TopLevelElement::Item(i) => visitor.visit_item(&self.code.items[i]),
                };
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

fn within_locations(loc: LineColumn, start: LineColumn, end: LineColumn) -> bool {
    (start.line < loc.line || (start.line == loc.line && start.column <= loc.column))
        && (loc.line < end.line || (loc.line == end.line && loc.column <= end.column))
}
