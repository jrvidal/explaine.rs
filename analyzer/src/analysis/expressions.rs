use super::NodeAnalyzer;
use crate::help::{LoopOf, ReturnOf};
use crate::{
    syn_wrappers::{Syn, SynKind},
    HelpItem,
};
use quote::ToTokens;
use syn::spanned::Spanned;

impl<'a> NodeAnalyzer<'a> {
    // EXAMPLE
    // impl<P: Deref<Target: Eq>> Eq for Pin<P> {}
    pub(super) fn visit_expr_array(&mut self, node: &syn::ExprArray) {
        if !self.has_ancestor(2, SynKind::ExprReference) {
            self.set_help(node, HelpItem::ExprArray)
        }
    }
    pub(super) fn visit_expr_assign(&mut self, node: &syn::ExprAssign) {
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssign);
    }
    pub(super) fn visit_expr_assign_op(&mut self, node: &syn::ExprAssignOp) {
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssignOp);
    }
    pub(super) fn visit_expr_async(&mut self, node: &syn::ExprAsync) {
        token![self, some node.capture, ExprAsyncMove];
        return self.set_help(node, HelpItem::ExprAsync);
    }
    pub(super) fn visit_expr_await(&mut self, node: &syn::ExprAwait) {
        token![self, node.await_token, ExprAwait];
    }
    pub(super) fn visit_expr_box(&mut self, node: &syn::ExprBox) {
        return self.set_help(node, HelpItem::ExprBox);
    }
    pub(super) fn visit_expr_break(&mut self, node: &syn::ExprBreak) {
        return self.set_help(
            &node,
            HelpItem::ExprBreak {
                expr: node.expr.is_some(),
                label: node.label.as_ref().map(|l| l.to_string()),
            },
        );
    }
    pub(super) fn visit_expr_cast(&mut self, node: &syn::ExprCast) {
        token![self, node.as_token, AsCast];
    }
    pub(super) fn visit_expr_closure(&mut self, node: &syn::ExprClosure) {
        token![self, node.or1_token, ExprClosureArguments];
        token![self, node.or2_token, ExprClosureArguments];
        token![self, some node.asyncness, ExprClosureAsync];
        token![self, some node.capture, ExprClosureMove];
        token![self, some node.movability, ExprClosureStatic];
        return self.set_help(node, HelpItem::ExprClosure);
    }
    pub(super) fn visit_expr_continue(&mut self, node: &syn::ExprContinue) {
        self.set_help(
            node,
            HelpItem::ExprContinue {
                label: node.label.as_ref().map(|l| l.to_string()),
            },
        );
    }
    pub(super) fn visit_expr_field(&mut self, node: &syn::ExprField) {
        if let syn::Member::Unnamed(..) = node.member {
            return self.set_help_between(
                node.dot_token.span(),
                node.member.span(),
                HelpItem::ExprUnnamedField,
            );
        }
    }
    pub(super) fn visit_expr_for_loop_first_pass(&mut self, node: &syn::ExprForLoop) {
        token![self, node.for_token, ExprForLoopToken];
        token![self, node.in_token, ExprForLoopToken];
        if self.within(&node.pat) {
            match &node.pat {
                syn::Pat::Ident(syn::PatIdent {
                    ident, mutability, ..
                }) => {
                    return self.set_help(
                        &node.pat,
                        HelpItem::ForLoopLocal {
                            mutability: mutability.is_some(),
                            ident: Some(ident.to_string()),
                        },
                    );
                }
                syn::Pat::Wild(..) => {
                    return self.set_help(
                        &node.pat,
                        HelpItem::ForLoopLocal {
                            mutability: false,
                            ident: None,
                        },
                    );
                }
                _ => {}
            }
        }
    }
    pub(super) fn visit_expr_if(&mut self, node: &syn::ExprIf) {
        if let syn::Expr::Let(syn::ExprLet { let_token, .. }) = *node.cond {
            if self.between_spans(node.if_token.span(), let_token.span()) {
                return self.set_help_between(
                    node.if_token.span(),
                    let_token.span(),
                    HelpItem::ExprIfLet,
                );
            }
        } else {
            token![self, node.if_token, ExprIf];
        };
        if let Some((else_token, _)) = node.else_branch {
            token![self, else_token, Else];
        }
    }
    pub(super) fn visit_expr_index(&mut self, node: &syn::ExprIndex) {
        let range = if let syn::Expr::Range(..) = &*node.index {
            true
        } else {
            false
        };

        return self.set_help(node, HelpItem::ExprIndex { range });
    }
    pub(super) fn visit_expr_loop(&mut self, node: &syn::ExprLoop) {
        token![self, node.loop_token, ExprLoopToken];
    }
    pub(super) fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
        token![self, node.match_token, ExprMatchToken];
    }
    pub(super) fn visit_expr_range(&mut self, node: &syn::ExprRange) {
        let from = node.from.is_some();
        let to = node.to.is_some();
        return match node.limits {
            syn::RangeLimits::HalfOpen(..) => {
                self.set_help(node, HelpItem::ExprRangeHalfOpen { from, to })
            }
            syn::RangeLimits::Closed(..) => {
                self.set_help(node, HelpItem::ExprRangeClosed { from, to })
            }
        };
    }
    pub(super) fn visit_expr_reference(&mut self, node: &syn::ExprReference) {
        let item = if let syn::Expr::Array(_) = &*node.expr {
            HelpItem::ExprArraySlice
        } else {
            HelpItem::ExprReference {
                mutable: node.mutability.is_some(),
            }
        };

        return self.set_help(node, item);
    }
    pub(super) fn visit_expr_repeat(&mut self, node: &syn::ExprRepeat) {
        return self.set_help(
            node,
            HelpItem::ExprRepeat {
                len: (&*node.len).to_token_stream().to_string(),
            },
        );
    }
    pub(super) fn visit_expr_return(&mut self, node: &syn::ExprReturn) {
        let of = self
            .ancestors
            .iter()
            .rev()
            .find_map(|(_, node)| match node {
                Syn::ItemFn(_) => Some(ReturnOf::Function),
                Syn::ImplItemMethod(_) => Some(ReturnOf::Method),
                Syn::TraitItemMethod(_) => Some(ReturnOf::Method),
                Syn::ExprClosure(_) => Some(ReturnOf::Closure),
                Syn::ExprAsync(_) => Some(ReturnOf::AsyncBlock),
                _ => None,
            })
            .unwrap_or(ReturnOf::Function);
        token![self, node.return_token, *HelpItem::ExprReturn { of }];
    }
    pub(super) fn visit_expr_struct(&mut self, node: &syn::ExprStruct) {
        if let Some((dot2_token, rest)) = node
            .dot2_token
            .and_then(|t| node.rest.as_ref().map(|r| (t, r)))
        {
            let start = dot2_token.span();
            let end = rest.span();
            if self.between_spans(start, end) {
                return self.set_help_between(start, end, HelpItem::ExprStructRest);
            }
        }

        // TODO: see [HITBOX]. Used to have only the path as clickable
        return self.set_help(node, HelpItem::ExprStruct);
    }
    pub(super) fn visit_expr_try(&mut self, node: &syn::ExprTry) {
        token![self, node.question_token, ExprTryQuestionMark];
    }
    pub(super) fn visit_expr_try_block(&mut self, node: &syn::ExprTryBlock) {
        token![self, node.try_token, ExprTryBlock];
    }
    pub(super) fn visit_expr_tuple(&mut self, node: &syn::ExprTuple) {
        return self.set_help(
            node,
            if node.elems.is_empty() {
                HelpItem::ExprUnitTuple
            } else {
                HelpItem::ExprTuple {
                    single_comma: node.elems.len() == 1 && node.elems.trailing_punct(),
                }
            },
        );
    }
    pub(super) fn visit_expr_type(&mut self, node: &syn::ExprType) {
        return self.set_help(node, HelpItem::ExprType);
    }
    pub(super) fn visit_expr_unsafe(&mut self, node: &syn::ExprUnsafe) {
        token![self, node.unsafe_token, ExprUnsafe];
    }
    pub(super) fn visit_expr_while(&mut self, node: &syn::ExprWhile) {
        let let_token = if let syn::Expr::Let(syn::ExprLet { let_token, .. }) = *node.cond {
            Some(let_token)
        } else {
            None
        };

        token![self, some node.label, * HelpItem::Label {
            loop_of: if let_token.is_some() { LoopOf::WhileLet } else { LoopOf::While },
        }];

        match let_token {
            Some(let_token) => {
                token![self, node.while_token => let_token, ExprWhileLet];
            }
            None => {
                token![self, node.while_token, ExprWhile];
            }
        }
    }
    pub(super) fn visit_expr_yield(&mut self, node: &syn::ExprYield) {
        token![self, node.yield_token, ExprYield];
    }
}
