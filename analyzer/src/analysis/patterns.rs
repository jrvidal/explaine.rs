use super::NodeAnalyzer;
use crate::help::HelpItem;
use crate::help::{BindingOf, RestOf};
use crate::syn_wrappers::{Syn, SynKind};
use syn::spanned::Spanned;

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn visit_pat_box(&mut self, node: &syn::PatBox) {
        token![self, node.box_token, PatBox];
    }
    pub(super) fn visit_pat_ident(&mut self, node: &syn::PatIdent) {
        if let Some((at_token, subpat)) = &node.subpat {
            if self.between(at_token, subpat) {
                return self.set_help(
                    node,
                    HelpItem::PatIdentSubPat {
                        ident: node.ident.to_string(),
                    },
                );
            }
        }

        if node.mutability.is_some() && self.has_ancestor(3, SynKind::FnArg) {
            return self.set_help(
                node,
                HelpItem::PatIdentMutableArg {
                    ident: node.ident.to_string(),
                },
            );
        }

        let item = HelpItem::PatIdent {
            mutability: node.mutability.is_some(),
            by_ref: node.by_ref.is_some(),
            ident: node.ident.to_string(),
        };
        match (node.by_ref, node.mutability) {
            (Some(by_ref), Some(mutability)) => token![self, by_ref => mutability, * item],
            (Some(by_ref), None) => token![self, by_ref, *item],
            (None, Some(mutability)) => token![self, mutability, *item],
            _ => {}
        }
    }
    pub(super) fn visit_pat_or(&mut self, node: &syn::PatOr) {
        token![self, some node.leading_vert, PatOrLeading];
        for pair in node.cases.pairs() {
            token![self, some pair.punct(), PatOr];
        }
    }
    pub(super) fn visit_pat_range(&mut self, node: &syn::PatRange) {
        return self.set_help(
            node,
            HelpItem::PatRange {
                closed: if let syn::RangeLimits::Closed(..) = node.limits {
                    true
                } else {
                    false
                },
            },
        );
    }
    pub(super) fn visit_pat_struct(&mut self, node: &syn::PatStruct) {
        if node.fields.is_empty() {
            token![self, some node.dot2_token, * HelpItem::PatStruct {
                empty: true,
                bindings: pattern_bindings(&self)
            }];
        }
        token![self, some node.dot2_token, * HelpItem::PatRest {
            of: RestOf::Struct
        }];

        return self.set_help(
            node,
            HelpItem::PatStruct {
                empty: false,
                bindings: pattern_bindings(&self),
            },
        );
    }
    pub(super) fn visit_pat_tuple(&mut self, node: &syn::PatTuple) {
        if get_ancestor![self, PatTupleStruct, 1].is_some() {
            return;
        }

        if let Some(pat @ syn::Pat::Rest(..)) = node.elems.last() {
            token![self, pat, *HelpItem::PatRest { of: RestOf::Tuple }];
        }

        let item = if node.elems.is_empty() {
            HelpItem::PatUnit
        } else {
            HelpItem::PatTuple {
                bindings: pattern_bindings(&self),
                single_comma: node.elems.len() == 1 && node.elems.trailing_punct(),
            }
        };
        return self.set_help(node, item);
    }
    pub(super) fn visit_pat_tuple_struct(&mut self, node: &syn::PatTupleStruct) {
        if let Some(pat @ syn::Pat::Rest(..)) = node.pat.elems.last() {
            token![
                self,
                pat,
                *HelpItem::PatRest {
                    of: RestOf::TupleStruct
                }
            ];
        }

        return self.set_help(
            node,
            HelpItem::PatTupleStruct {
                bindings: pattern_bindings(&self),
            },
        );
    }
    pub(super) fn visit_pat_wild(&mut self, node: &syn::PatWild) {
        let last_arm = match (
            get_ancestor![self, ExprMatch, 3],
            get_ancestor![self, Arm, 2],
        ) {
            (Some(match_expr), Some(arm)) => match_expr
                .arms
                .last()
                .map(|match_arm| std::ptr::eq(match_arm, arm))
                .unwrap_or(false),
            _ => false,
        };

        return self.set_help(node, HelpItem::PatWild { last_arm });
    }
}

fn pattern_bindings(analyzer: &NodeAnalyzer) -> Option<BindingOf> {
    if analyzer.has_ancestor(2, SynKind::Local)
        || (analyzer.has_ancestor(4, SynKind::Local) && analyzer.has_ancestor(2, SynKind::PatType))
    {
        return Some(BindingOf::Let);
    }
    if analyzer.has_ancestor(3, SynKind::FnArg) {
        return Some(BindingOf::Arg);
    }

    if analyzer.has_ancestor(2, SynKind::ExprForLoop) {
        return Some(BindingOf::ForLoop);
    }

    None
}
