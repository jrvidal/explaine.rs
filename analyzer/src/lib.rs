use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use std::any::Any;
use syn::spanned::Spanned;
use syn::visit::Visit;

mod help;

#[cfg(test)]
mod tests;

use crate::help::*;
pub use help::HelpItem;

#[cfg(feature = "dev")]
trait DynAncestor {
    fn as_any(&self) -> &dyn Any;
    fn as_debug(&self) -> &dyn Debug;
}

#[cfg(not(feature = "dev"))]
trait DynAncestor {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "dev")]
impl<T: Debug + Any> DynAncestor for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
    fn as_debug(&self) -> &dyn Debug {
        self as &dyn Debug
    }
}

#[cfg(not(feature = "dev"))]
impl<T: Any> DynAncestor for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

pub struct IntersectionVisitor<'ast> {
    /// The location for the help request
    location: LineColumn,
    /// The contextual help we return
    help: HelpItem,
    /// The location of the contextual help
    item_location: (LineColumn, LineColumn),
    /// The attributes of the parent element, only trust in `visit_attribute`
    attributes: &'ast [syn::Attribute],
    /// The ancestors of the current element
    ancestors: Vec<&'ast dyn DynAncestor>,
    #[cfg(feature = "dev")]
    log: fn(&str),
}

pub struct VisitorResult {
    pub help: HelpItem,
    pub item_location: (LineColumn, LineColumn),
}

impl<'ast> IntersectionVisitor<'ast> {
    pub fn new(location: LineColumn, #[cfg(feature = "dev")] log: fn(&str)) -> Self {
        Self {
            location,
            item_location: (location, location),
            help: HelpItem::Unknown,
            attributes: &[],
            ancestors: vec![],
            #[cfg(feature = "dev")]
            log,
        }
    }

    fn settled(&self) -> bool {
        match self.help {
            HelpItem::Unknown => false,
            _ => true,
        }
    }

    pub fn visit(mut self, file: &'ast syn::File) -> VisitorResult {
        self.visit_file(file);

        VisitorResult {
            help: self.help,
            item_location: self.item_location,
        }
    }

    pub fn visit_element(mut self, file: &'ast syn::File, idx: usize) -> VisitorResult {
        self.attributes = &file.attrs[..];
        self.ancestors.push(file);

        if idx < file.attrs.len() {
            self.visit_attribute(&file.attrs[idx]);
        } else {
            self.visit_item(&file.items[idx - file.attrs.len()]);
        }
        VisitorResult {
            help: self.help,
            item_location: self.item_location,
        }
    }

    fn within<S: Spanned>(&self, item: S) -> bool {
        let span = item.span();
        self.between_spans(span, span)
    }

    fn between<S: Spanned, T: Spanned>(&self, start: &S, end: &T) -> bool {
        self.between_spans(start.span(), end.span())
    }

    fn between_spans(&self, start: Span, end: Span) -> bool {
        self.between_locations(start.start(), end.end())
    }

    fn between_locations(&self, start: LineColumn, end: LineColumn) -> bool {
        let loc = self.location;
        within_locations(loc, start, end)
    }

    fn set_help<S: Spanned + ?Sized>(&mut self, item: &S, help: HelpItem) {
        self.set_help_between(item.span(), item.span(), help);
    }

    fn set_help_between(
        &mut self,
        start: proc_macro2::Span,
        end: proc_macro2::Span,
        help: HelpItem,
    ) {
        self.help = help;
        self.item_location = (start.start(), end.end());
    }

    fn get_ancestor<T: 'static>(&self, idx: usize) -> Option<&'ast T> {
        self.ancestors
            .get(self.ancestors.len() - idx)
            .and_then(|any| (*any).as_any().downcast_ref())
    }

    fn has_ancestor<T: 'static>(&self, idx: usize) -> bool {
        self.ancestors
            .get(self.ancestors.len() - idx)
            .map(|any| (*any).as_any().is::<T>())
            .unwrap_or(false)
    }
}

macro_rules! method {
    (@_spancheck, $self:ident, $node:ident) => {{
        if !$self.within(&$node) {
            return;
        }
    }};
    (@_attrs_spancheck, $self:ident, $node:ident) => {{
        method![@_spancheck, $self, $node];

        for attr in $node.attrs.iter() {
            if $self.within(attr) {
                method![@_push_ancestor, $self, $node];
                $self.attributes = &$node.attrs[..];
                $self.visit_attribute(attr);
                method![@_pop_ancestor, $self, $node];
                return;
            }
        }
    }};
    (@_nospancheck, $self:ident, $node:ident) => {};
    (@_debug, $name:ident, $self:ident, $node:ident) => {{
        #[cfg(feature = "dev")]
        {
            ($self.log)(stringify!($name));
        }
    }};
    (@_push_ancestor, $self:ident, $node:ident) => {
        $self.ancestors.push($node);
    };
    (@_pop_ancestor, $self:ident, $node:ident) => {
        let _ = $self.ancestors.pop();
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body $body:expr => $after:expr, @ $spancheck:ident) => {
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@_debug, $name, $self, $node];
            method![@$spancheck, $self, $node];
            $body;
            method![@_push_ancestor, $self, $node];
            syn::visit::$name($self, $node);
            method![@_pop_ancestor, $self, $node];
            $after;
        }
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body @terminal $body:expr, @ $spancheck:ident) => {
        #[allow(unreachable_code)]
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@_debug, $name, $self, $node];
            method![@$spancheck, $self, $node];
            $body;
            unreachable!(stringify!($name));
        }
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body];
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal $body];
    };
    // "Public"
    ($name:ident($self:ident, $node:ident: $ty:path)) => {
        method![$name($self, $node: $ty) { () }];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) $body:expr => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body => $after, @_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body () => $after, @_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body => (), @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path)) => {
        method![@_impl, $name, $self, $node, $ty, @_body () => (), @_attrs_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body => (), @_attrs_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal $body, @_attrs_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) $body:expr => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body => $after, @_attrs_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body () => $after, @_attrs_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal $body, @_spancheck];
    };
    // Sometimes the naive `between()` test is misleading, so we better skip it
    (@nospancheck $name:ident($self:ident, $node:ident: $ty:path)) => {
        method![@_impl, $name, $self, $node, $ty, @_body { () } => (), @_nospancheck];
    };
}

macro_rules! token {
    ($self:expr, $token:expr, $item:ident) => {
        token![$self, $token, *HelpItem::$item];
    };
    ($self:expr, $start:expr => $end:expr, $item:ident) => {
        token![$self, $start => $end, * HelpItem::$item];
    };
    ($self:expr, $start:expr => $end:expr, * $item:expr) => {
        if $self.between(&$start, &$end) {
            return $self.set_help_between($start.span(), $end.span(), $item);
        }
    };
    ($self:expr, $token:expr, * $item:expr) => {
        if $self.within(&$token) {
            return $self.set_help(&$token, $item);
        }
    };
    ($self:expr, some $token:expr, $item:ident) => {
        token![$self, some $token, * HelpItem::$item];
    };
    ($self:expr, some $token:expr, * $item:expr) => {
        if let Some(ref inner) = &$token {
            token![$self, inner, * $item];
        }
    };
}

impl<'ast> Visit<'ast> for IntersectionVisitor<'ast> {
    // OMITTED: handled upstream
    // method![visit_abi(self, node: syn::Abi)];
    method![visit_angle_bracketed_generic_arguments(
        self,
        node: syn::AngleBracketedGenericArguments
    ) => {
        if self.settled() {
            return;
        }
        if node.colon2_token.is_some() {
            return self.set_help(node, HelpItem::Turbofish);
        }
    }];
    method![@attrs visit_arm(self, node: syn::Arm) {
        token![self, node.fat_arrow_token, FatArrow];
        if let Some((if_token, _)) = node.guard {
            token![self, if_token, ArmIfGuard];
        }
    }];
    // OMITTED: handled in visit_attribute
    // method![visit_attr_style(self, node: syn::AttrStyle)];
    method![visit_attribute(self, node: syn::Attribute) @terminal {
        let outer = outer_attr(node);
        if node.path.is_ident("doc") {
            let bounds = if self.attributes.len() > 0 {
                let this_idx = self.attributes
                    .iter()
                    .enumerate()
                    .find(|(_, attr)| std::ptr::eq(node, *attr))
                    .expect("attr in list")
                    .0;

                let last = self.attributes
                    .iter()
                    .enumerate()
                    .filter(|&(i, attr)| {
                        i >= this_idx && attr.path.is_ident("doc") && outer_attr(attr) == outer
                    })
                    .last()
                    .expect("last")
                    .1;

                let start = self.attributes
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|&(i, attr)| {
                        i <= this_idx && !(attr.path.is_ident("doc") && outer_attr(attr) == outer)
                    })
                    .next()
                    .map(|(i, _)| &self.attributes[i - 1])
                    .unwrap_or(&self.attributes[0]);

                Some((start.span(), last.span()))
            } else {
                None
            };
            if let Some((start, end)) = bounds {
                return self.set_help_between(start, end, HelpItem::DocBlock { outer });
            }
        }
        return self.set_help(&node, HelpItem::Attribute { outer });
    }];
    method![@attrs visit_bare_fn_arg(self, node: syn::BareFnArg)];
    method![visit_bin_op(self, node: syn::BinOp) @terminal {
        use syn::BinOp::*;
        let item = match node {
            Add(..) => HelpItem::AddBinOp,
            Sub(..) => HelpItem::SubBinOp,
            Mul(..) => HelpItem::MulBinOp,
            Div(..) => HelpItem::DivBinOp,
            Rem(..) => HelpItem::RemBinOp,
            And(..) => HelpItem::AndBinOp,
            Or(..) => HelpItem::OrBinOp,
            BitXor(..) => HelpItem::BitXorBinOp,
            BitAnd(..) => HelpItem::BitAndBinOp,
            BitOr(..) => HelpItem::BitOrBinOp,
            Shl(..) => HelpItem::ShlBinOp,
            Shr(..) => HelpItem::ShrBinOp,
            Eq(..) => HelpItem::EqBinOp,
            Lt(..) => HelpItem::LtBinOp,
            Le(..) => HelpItem::LeBinOp,
            Ne(..) => HelpItem::NeBinOp,
            Ge(..) => HelpItem::GeBinOp,
            Gt(..) => HelpItem::GtBinOp,
            AddEq(..) => HelpItem::AddEqBinOp,
            SubEq(..) => HelpItem::SubEqBinOp,
            MulEq(..) => HelpItem::MulEqBinOp,
            DivEq(..) => HelpItem::DivEqBinOp,
            RemEq(..) => HelpItem::RemEqBinOp,
            BitXorEq(..) => HelpItem::BitXorEqBinOp,
            BitAndEq(..) => HelpItem::BitAndEqBinOp,
            BitOrEq(..) => HelpItem::BitOrEqBinOp,
            ShlEq(..) => HelpItem::ShlEqBinOp,
            ShrEq(..) => HelpItem::ShrEqBinOp,
        };

        return self.set_help(node, item);
    }];
    method![visit_binding(self, node: syn::Binding) => {
        if !self.settled() {
            return self.set_help(node, HelpItem::Binding {
                ident: node.ident.to_string()
            });
        }
    }];
    method![visit_block(self, node: syn::Block)];
    method![visit_bound_lifetimes(self, node: syn::BoundLifetimes) @terminal {
        return self.set_help(&node, HelpItem::BoundLifetimes);
    }];
    method![@attrs visit_const_param(self, node: syn::ConstParam) {
        token![self, node.const_token, ConstParam];
    }];
    // EXAMPLE
    // impl<P: Deref<Target: Eq>> Eq for Pin<P> {}
    method![visit_constraint(self, node: syn::Constraint)];

    // OMITTED: unreachable from File
    // method![visit_data(self, node: syn::Data)];
    // method![visit_data_enum(self, node: syn::DataEnum)];
    // method![visit_data_struct(self, node: syn::DataStruct)];
    // method![visit_data_union(self, node: syn::DataUnion)];
    // fn visit_derive_input(&mut self, _: &'ast syn::DeriveInput) {}

    method![visit_expr(self, node: syn::Expr)];
    method![@attrs visit_expr_array(self, node: syn::ExprArray) => {
        if self.settled() {
            return;
        }

        if !self.has_ancestor::<syn::ExprReference>(2) {
            self.set_help(node, HelpItem::ExprArray)
        }
    }];
    method![@attrs visit_expr_assign(self, node: syn::ExprAssign) => {
        if self.settled() {
            return;
        }
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssign);
    }];
    method![@attrs visit_expr_assign_op(self, node: syn::ExprAssignOp) => {
        if self.settled() {
            return;
        }
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssignOp);
    }];
    method![@attrs visit_expr_async(self, node: syn::ExprAsync) {
        token![self, some node.capture, ExprAsyncMove];
    } => {
        if self.settled() {
            return;
        }
        return self.set_help(node, HelpItem::ExprAsync);
    }];
    method![@attrs visit_expr_await(self, node: syn::ExprAwait) {
        token![self, node.await_token, ExprAwait];
    }];
    method![@attrs visit_expr_binary(self, node: syn::ExprBinary)];
    method![@attrs visit_expr_block(self, node: syn::ExprBlock)];
    method![@attrs visit_expr_box(self, node: syn::ExprBox)];
    method![@attrs visit_expr_break(self, node: syn::ExprBreak) => {
        if self.settled() {
            return;
        }
        return self.set_help(&node, HelpItem::ExprBreak {
            expr: node.expr.is_some(),
            label: node.label.as_ref().map(|l| l.to_string())
        });
    }];
    method![@attrs visit_expr_call(self, node: syn::ExprCall)];
    method![@attrs visit_expr_cast(self, node: syn::ExprCast) {
        token![self, node.as_token, AsCast];
    }];
    method![@attrs visit_expr_closure(self, node: syn::ExprClosure) {
        token![self, node.or1_token, ExprClosureArguments];
        token![self, node.or2_token, ExprClosureArguments];
        token![self, some node.asyncness, ExprClosureAsync];
        token![self, some node.capture, ExprClosureMove];
        token![self, some node.movability, ExprClosureStatic];
    } => {
        if self.settled() {
            return;
        }

        return self.set_help(node, HelpItem::ExprClosure);
    }];
    method![@attrs visit_expr_continue(self, node: syn::ExprContinue) {
        self.set_help(node, HelpItem::ExprContinue { label: node.label.as_ref().map(|l| l.to_string()) });
    }];
    method![@attrs visit_expr_field(self, node: syn::ExprField) => {
        if self.settled() {
            return;
        }

        if let syn::Member::Unnamed(..) = node.member {
            return self.set_help_between(node.dot_token.span(), node.member.span(), HelpItem::ExprUnnamedField);
        }
    }];
    method![@attrs visit_expr_for_loop(self, node: syn::ExprForLoop) {
        token![self, node.for_token, ExprForLoopToken];
        token![self, node.in_token, ExprForLoopToken];
        if self.within(&node.pat) {
            match &node.pat {
                syn::Pat::Ident(syn::PatIdent { ident, mutability, .. }) => {
                    return self.set_help(&node.pat, HelpItem::ForLoopLocal {
                        mutability: mutability.is_some(),
                        ident: Some(ident.to_string())
                    });
                },
                syn::Pat::Wild(..) => {
                    return self.set_help(&node.pat, HelpItem::ForLoopLocal{
                        mutability: false,
                        ident: None
                    });
                }
                _ => {}
            }
        }
    }];
    method![@attrs visit_expr_group(self, node: syn::ExprGroup)];
    method![@attrs visit_expr_if(self, node: syn::ExprIf) {
        if let syn::Expr::Let(syn::ExprLet { let_token, .. }) = *node.cond {
            if self.between_spans(node.if_token.span(), let_token.span()) {
                return self.set_help_between(node.if_token.span(), let_token.span(), HelpItem::ExprIfLet);
            }
        } else {
            token![self, node.if_token, ExprIf];
        };
        if let Some((else_token, _)) = node.else_branch {
            token![self, else_token, Else];
        }
    }];
    method![@attrs visit_expr_index(self, node: syn::ExprIndex) => {
        if self.settled() {
            return;
        }
        let range = if let syn::Expr::Range(..) = &*node.index {
            true
        } else {
            false
        };

        return self.set_help(node, HelpItem::ExprIndex {
            range
        });
    }];
    method![@attrs visit_expr_let(self, node: syn::ExprLet)];
    method![@attrs visit_expr_lit(self, node: syn::ExprLit)];
    method![@attrs visit_expr_loop(self, node: syn::ExprLoop) {
        token![self, node.loop_token, ExprLoopToken];
    }];
    method![@attrs visit_expr_macro(self, node: syn::ExprMacro)];
    method![@attrs visit_expr_match(self, node: syn::ExprMatch) {
        token![self, node.match_token, ExprMatchToken];
    }];
    method![@attrs visit_expr_method_call(self, node: syn::ExprMethodCall)];
    method![@attrs visit_expr_paren(self, node: syn::ExprParen)];
    method![@attrs visit_expr_path(self, node: syn::ExprPath)];
    method![@attrs visit_expr_range(self, node: syn::ExprRange) @terminal {
        let from = node.from.is_some();
        let to = node.to.is_some();
        match node.limits {
            syn::RangeLimits::HalfOpen(..) => return self.set_help(node, HelpItem::ExprRangeHalfOpen { from, to }),
            syn::RangeLimits::Closed(..) => return self.set_help(node, HelpItem::ExprRangeClosed { from, to }),
        }
    }];
    method![@attrs visit_expr_reference(self, node: syn::ExprReference) => {
        if self.settled() {
            return;
        }

        let item = if let syn::Expr::Array(_) = &*node.expr {
            HelpItem::ExprArraySlice
        } else {
            HelpItem::ExprReference {
                mutable: node.mutability.is_some()
            }
        };

        return self.set_help(node, item);
    }];
    method![@attrs visit_expr_repeat(self, node: syn::ExprRepeat) => {
        if !self.settled() {
            return self.set_help(node, HelpItem::ExprRepeat {
                len: (&*node.len).to_token_stream().to_string()
            });
        }
    }];
    method![@attrs visit_expr_return(self, node: syn::ExprReturn) {
        token![self, node.return_token, ExprReturn];
    }];
    method![@attrs visit_expr_struct(self, node: syn::ExprStruct) => {
        if self.settled() {
            return;
        }

        if self.within(&node.path) {
            return self.set_help(node, HelpItem::ExprStruct);
        }

        if let Some((dot2_token, rest)) = node
            .dot2_token
            .and_then(|t| node.rest.as_ref().map(|r| (t, r)))
        {
            if self.between_spans(dot2_token.span(), rest.span()) {
                return self.set_help_between(
                    dot2_token.span(),
                    rest.span(),
                    HelpItem::ExprStructRest,
                );
            }
        }
    }];
    method![@attrs visit_expr_try(self, node: syn::ExprTry) {
        token![self, node.question_token, ExprTryQuestionMark];
    }];
    method![@attrs visit_expr_try_block(self, node: syn::ExprTryBlock) {
        token![self, node.try_token, ExprTryBlock];
    }];
    method![@attrs visit_expr_tuple(self, node: syn::ExprTuple) => {
        if !self.settled() {
            return self.set_help(node, if node.elems.is_empty() { HelpItem::ExprUnitTuple } else { HelpItem::ExprTuple });
        }
    }];
    method![@attrs visit_expr_type(self, node: syn::ExprType) => {
        if !self.settled() {
            return self.set_help(node, HelpItem::ExprType);
        }
    }];
    method![@attrs visit_expr_unary(self, node: syn::ExprUnary)];
    method![@attrs visit_expr_unsafe(self, node: syn::ExprUnsafe) {
        token![self, node.unsafe_token, ExprUnsafe];
    }];
    method![@attrs visit_expr_while(self, node: syn::ExprWhile) {
        let while_let = if let syn::Expr::Let(..) = *node.cond {
            true
        } else {
            false
        };

        token![self, some node.label, * HelpItem::Label {
            loop_of: if while_let { LoopOf::WhileLet } else { LoopOf::While },
        }];

        token![self, node.while_token, * if while_let { HelpItem::ExprWhileLet  } else { HelpItem::ExprWhile }];
    }];
    method![@attrs visit_expr_yield(self, node: syn::ExprYield) {
        token![self, node.yield_token, ExprYield];
    }];
    method![@attrs visit_field(self, node: syn::Field) => {
        if self.settled() {
            return;
        }

        let field_data = loop {
            if let Some(variant) = self.get_ancestor::<syn::Variant>(3) {
                break Some((FieldOf::Variant, variant.ident.to_string()));
            }

            if let Some(item_struct) = self.get_ancestor::<syn::ItemStruct>(3) {
                break Some((FieldOf::Struct, item_struct.ident.to_string()));
            }

            if let Some(item_union) = self.get_ancestor::<syn::ItemUnion>(2) {
                break Some((FieldOf::Union, item_union.ident.to_string()));
            }

            break None;
        };

        if let Some((field_of, of_name)) = field_data {
            return self.set_help(node, HelpItem::Field {
                name: node.ident.as_ref().map(|id| id.to_string()),
                of: field_of,
                of_name
            });
        }

    }];
    method![@attrs visit_field_pat(self, node: syn::FieldPat) => {
        if self.settled() {
            return;
        }

        if let (syn::Member::Unnamed(..), syn::Pat::Ident(syn::PatIdent { ident, .. })) =
            (&node.member, &*node.pat)
        {
            return self.set_help(
                node,
                HelpItem::FieldPatUnnamed {
                    ident: ident.to_string(),
                },
            );
        }


        if let (None, syn::Pat::Ident(syn::PatIdent { ident, .. })) = (node.colon_token, &*node.pat) {
            return self.set_help(node, HelpItem::FieldPatShorthand {
                ident: ident.to_string()
            });
        }
    }];
    method![@attrs visit_field_value(self, node: syn::FieldValue) {
        match (node.colon_token, &node.member) {
            (None, syn::Member::Named(ident)) => {
                return self.set_help(node, HelpItem::FieldValueShorthand {
                    name: ident.to_string()
                });
            },
            _ => {}
        }
    } => {
        if self.settled() {
            return
        }

        if let syn::Member::Unnamed(..) = node.member {
            return self.set_help(node, HelpItem::FieldUnnamedValue);
        }
    }];
    method![visit_fields(self, node: syn::Fields)];
    method![visit_fields_named(self, node: syn::FieldsNamed)];
    method![visit_fields_unnamed(self, node: syn::FieldsUnnamed)];
    fn visit_file(&mut self, node: &'ast syn::File) {
        // TODO: handle shebang in exploration
        // TODO: assign proper span to shebang
        token![self, some node.shebang, Shebang];
        self.ancestors.push(node);

        for attr in &node.attrs {
            if self.within(attr) {
                self.attributes = &node.attrs[..];
                self.visit_attribute(attr);
                return;
            }
        }

        for item in &node.items {
            if self.within(item) {
                self.visit_item(item);
                return;
            }
        }
    }
    method![visit_fn_arg(self, node: syn::FnArg) {
        loop {
            let sig = if let Some(sig) = self.get_ancestor::<syn::Signature>(1) {
                sig
            } else {
                break;
            };

            let is_first = sig
                .inputs
                .first()
                .map(|arg| std::ptr::eq(arg, node))
                .unwrap_or(false);
            if !is_first {
                break;
            }

            let pat = if let syn::FnArg::Typed(pat_type) = node {
                pat_type
            } else {
                break;
            };

            let pat_ident = match &*pat.pat {
                syn::Pat::Ident(pat_ident) => pat_ident,
                _ => break,
            };

            let is_self = pat_ident.by_ref.is_none()
                && pat_ident.subpat.is_none()
                && pat_ident.ident == "self";

            if !is_self {
                break;
            }

            let mutability = pat_ident.mutability.is_some();

            match &*pat.ty {
                syn::Type::Path(type_path) if type_path.path.is_ident("Self") => {
                    return self.set_help(
                        node,
                        HelpItem::ValueSelf {
                            explicit: true,
                            mutability,
                        },
                    )
                }
                syn::Type::Reference(type_reference) => match &*type_reference.elem {
                    syn::Type::Path(type_ref_path) if type_ref_path.path.is_ident("Self") => {
                        return self.set_help(
                            node,
                            if type_reference.mutability.is_some() {
                                HelpItem::MutSelf {
                                    explicit: true,
                                    mutability,
                                }
                            } else {
                                HelpItem::RefSelf {
                                    explicit: true,
                                    mutability,
                                }
                            },
                        )
                    }
                    _ => {}
                },
                _ => return self.set_help(node, HelpItem::SpecialSelf { mutability }),
            }

            break;
        }
    }];
    method![visit_foreign_item(self, node: syn::ForeignItem)];
    method![@attrs visit_foreign_item_fn(self, node: syn::ForeignItemFn)];
    method![@attrs visit_foreign_item_macro(self, node: syn::ForeignItemMacro)];
    method![@attrs visit_foreign_item_static(
        self,
        node: syn::ForeignItemStatic
    ) {
        let end = node.mutability.as_ref().map(Spanned::span)
            .unwrap_or_else(|| node.static_token.span());

        if self.between_spans(node.static_token.span(), end) {
            return self.set_help_between(node.static_token.span(), end, if node.mutability.is_some() {
                HelpItem::StaticMut
            } else {
                HelpItem::Static
            });
        }
    }];
    method![@attrs visit_foreign_item_type(self, node: syn::ForeignItemType) {
        token![self, node.type_token, ForeignItemType];
    }];
    method![visit_generic_argument(self, node: syn::GenericArgument)];
    method![visit_generic_method_argument(
        self,
        node: syn::GenericMethodArgument
    )];
    method![visit_generic_param(self, node: syn::GenericParam)];
    method![@nospancheck visit_generics(self, node: syn::Generics)];
    method![visit_ident(self, node: syn::Ident) {
        let raw = node.to_string();

        let start = node.span().start();
        if raw.starts_with("r#") && self.between_locations(start, LineColumn {
            column: start.column + 2,
            ..start
        }) {
            return self.set_help(node, HelpItem::RawIdent);
        }
    }];
    method![visit_impl_item(self, node: syn::ImplItem)];
    method![@attrs visit_impl_item_const(self, node: syn::ImplItemConst) {
        token![self, node.const_token, ImplItemConst];
    }];
    method![@attrs visit_impl_item_macro(self, node: syn::ImplItemMacro)];
    method![@attrs visit_impl_item_method(self, node: syn::ImplItemMethod) {
        token![self, node.sig.ident, ImplItemMethod];
        if self.within(node.sig.fn_token) {
            if let Some(item_impl) = self.get_ancestor::<syn::ItemImpl>(2) {
                return self.set_help(&node.sig.fn_token, HelpItem::FnToken {
                    of: if item_impl.trait_.is_some() { FnOf::TraitMethod } else { FnOf::Method },
                    name: node.sig.ident.to_string()
                });
            }
        }
    }];
    method![@attrs visit_impl_item_type(self, node: syn::ImplItemType) {
        token![self, node.type_token, ImplItemType];
    }];
    method![visit_index(self, node: syn::Index)];
    method![visit_item(self, node: syn::Item)];
    method![@attrs visit_item_const(self, node: syn::ItemConst) {
        token![self, node.const_token, ItemConst];
    }];
    method![@attrs visit_item_enum(self, node: syn::ItemEnum) => {
        token![self, node.enum_token => node.ident, * HelpItem::ItemEnum {
            empty: node.variants.is_empty()
        }];
    }];
    method![@attrs visit_item_extern_crate(self, node: syn::ItemExternCrate) @terminal {
        if let Some((as_token, _)) = node.rename {
            token![self, as_token, AsRenameExternCrate];
        }
        let start = vis_span(&node.vis).unwrap_or_else(|| node.extern_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemExternCrate);
    }];
    method![@attrs visit_item_fn(self, node: syn::ItemFn) {
        token![self, node.sig.ident, ItemFn];
        token![self, node.sig.fn_token, * HelpItem::FnToken { of: FnOf::Function, name: node.sig.ident.to_string() }];
    }];
    method![@attrs visit_item_foreign_mod(self, node: syn::ItemForeignMod) {
        token![self, node.abi, ItemForeignModAbi];
    }];
    method![@attrs visit_item_impl(self, node: syn::ItemImpl) {
        token![self, some node.unsafety, ItemUnsafeImpl];
        if let Some((_, _, for_token)) = node.trait_ {
            token![self, for_token, ItemImplForTrait];
        }
        token![self, node.impl_token, * HelpItem::ItemImpl {
            trait_: node.trait_.is_some()
        }];
    }];
    method![@attrs visit_item_macro(self, node: syn::ItemMacro) {
        if let Some(ident) = &node.ident {
            if node.mac.path.is_ident("macro_rules") {
                return self.set_help(node, HelpItem::ItemMacroRules {
                    name: ident.to_string()
                })
            }
        }
    }];
    method![@attrs visit_item_macro2(self, node: syn::ItemMacro2)];
    method![@attrs visit_item_mod(self, node: syn::ItemMod) => {
        if self.settled() {
            return;
        }

        if node.content.is_some() {
            if self.between(&node.mod_token, &node.ident) {
                return self.set_help_between(node.mod_token.span(), node.ident.span(), HelpItem::ItemInlineMod);
            }
        } else {
            return self.set_help(&node, HelpItem::ItemExternMod);
        }
    }];
    method![@attrs visit_item_static(self, node: syn::ItemStatic) {
        let end = node.mutability.as_ref().map(Spanned::span)
            .unwrap_or_else(|| node.static_token.span());

        if self.between_spans(node.static_token.span(), end) {
            return self.set_help_between(node.static_token.span(), end, if node.mutability.is_some() {
                HelpItem::StaticMut
            } else {
                HelpItem::Static
            });
        }
    }];
    method![@attrs visit_item_struct(self, node: syn::ItemStruct) => {
        if self.settled() {
            return;
        }
        let unit = match node.fields {
            syn::Fields::Unit => true,
            _ => false
        };
        if self.between_spans(node.struct_token.span(), node.ident.span()) {
            return self.set_help_between(node.struct_token.span(), node.ident.span(), HelpItem::ItemStruct {
                unit,
                name: node.ident.to_string()
            });
        }
    }];
    method![@attrs visit_item_trait(self, node: syn::ItemTrait) {
        token![self, some node.unsafety, ItemUnsafeTrait];
        token![self, some node.auto_token, ItemAutoTrait];
        token![self, node.trait_token, ItemTrait];
        if let Some(colon_token) = node.colon_token {
            if self.within(colon_token) {
                let last = node.supertraits.last().map(|t| t.span()).unwrap_or(colon_token.span());
                return self.set_help_between(colon_token.span(), last, HelpItem::ItemTraitSupertraits);
            }
        }
    }];
    method![@attrs visit_item_trait_alias(self, node: syn::ItemTraitAlias) {
        token![self, node.trait_token, ItemTraitAlias];
    }];
    method![@attrs visit_item_type(self, node: syn::ItemType) {
        token![self, node.type_token => node.ident, ItemType];
    }];
    method![@attrs visit_item_union(self, node: syn::ItemUnion) {
        token![self, node.union_token, ItemUnion];
    }];
    method![@attrs visit_item_use(self, node: syn::ItemUse) {
        token![self, some node.leading_colon, PathLeadingColon];
    } => {
        if self.settled() {
            return;
        }
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemUse);
    }];
    method![visit_label(self, node: syn::Label) @terminal {
        let loop_of = if self.get_ancestor::<syn::ExprLoop>(1).is_some() {
            LoopOf::Loop
        } else if self.get_ancestor::<syn::ExprForLoop>(1).is_some() {
            LoopOf::For
        } else if self.get_ancestor::<syn::ExprBlock>(1).is_some() {
            LoopOf::Block
        } else {
            // Handled in ExprWhile
            return;
        };

        return self.set_help(&node, HelpItem::Label {
            loop_of
        });
    }];
    method![visit_lifetime(self, node: syn::Lifetime) {
        if node.ident == "static" {
            return self.set_help(node, HelpItem::StaticLifetime);
        }
    }];
    method![@attrs visit_lifetime_def(self, node: syn::LifetimeDef)];
    method![visit_lit(self, node: syn::Lit)];
    method![visit_lit_bool(self, node: syn::LitBool) @terminal {
        return self.set_help(node, if node.value { HelpItem::True } else { HelpItem:: False });
    }];
    method![visit_lit_byte(self, node: syn::LitByte) @terminal {
        return self.set_help(node, HelpItem::LitByte);
    }];
    method![visit_lit_byte_str(self, node: syn::LitByteStr) @terminal {
        let prefix = raw_string_literal(node.to_token_stream().to_string(), "br");
        let raw = prefix.is_some();
        return self.set_help(node, HelpItem::LitByteStr { raw, prefix });
    }];
    method![visit_lit_char(self, node: syn::LitChar) @terminal {
        return self.set_help(node, HelpItem::LitChar);
    }];
    method![visit_lit_float(self, node: syn::LitFloat) @terminal {
        let raw = node.to_string();
        let suffix = Some(node.suffix())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let separators = raw.chars().any(|c| c == '_');

        return self.set_help(node, HelpItem::LitFloat {
            suffix,
            separators
        });
    }];
    method![visit_lit_int(self, node: syn::LitInt) @terminal {
        let raw = node.to_string();

        let suffix = Some(node.suffix())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let separators = raw.chars().any(|c| c == '_');

        let (prefix, mode) = match raw.get(0..2) {
            prefix @ Some("0b") => (prefix, Some(IntMode::Binary)),
            prefix @ Some("0x") => (prefix, Some(IntMode::Hexadecimal)),
            prefix @ Some("0o") => (prefix, Some(IntMode::Octal)),
            _ => (None, None)
        };

        return self.set_help(node, HelpItem::LitInt {
            mode,
            separators,
            suffix,
            prefix: prefix.map(|s| s.to_string())
        });
    }];
    method![visit_lit_str(self, node: syn::LitStr) @terminal {
        let prefix = raw_string_literal(node.to_token_stream().to_string(), "r");
        let raw = prefix.is_some();
        return self.set_help(node, HelpItem::LitStr { raw, prefix });
    }];
    method![@attrs visit_local(self, node: syn::Local) {
        let ident_pat = match &node.pat {
            syn::Pat::Ident(pat) => Some(pat),
            syn::Pat::Type(syn::PatType { pat, .. }) => match &**pat {
                syn::Pat::Ident(pat_ident) => Some(pat_ident),
                _ => None
            },
            _ => None
        };


        match ident_pat {
            Some(syn::PatIdent { ident, mutability, ..}) => {
                token![self, node.let_token => ident, * HelpItem::Local {
                    mutability: mutability.is_some(),
                    ident: Some(ident.to_string())
                }];
            },
            _ => {
                token![self, node.let_token, * HelpItem::Local {
                    mutability: false,
                    ident: None
                }];
            }
        }

    }];
    method![visit_macro(self, node: syn::Macro) {
        if self.between_spans(node.path.span(), node.bang_token.span()) {
            return self.set_help_between(node.path.span(), node.bang_token.span(), HelpItem::Macro);
        }
        token![self, node.tokens, MacroTokens];
    }];
    // OMITTED: nothing interesting to say
    // method![visit_macro_delimiter(self, node: syn::MacroDelimiter)];
    method![visit_member(self, node: syn::Member)];
    // OMITTED: unreachable from File
    // method![visit_meta(self, node: syn::Meta)];
    // OMITTED: unreachable from File
    // method![visit_meta_list(self, node: syn::MetaList)];
    // OMITTED: unreachable from File
    // method![visit_meta_name_value(self, node: syn::MetaNameValue)];
    method![visit_method_turbofish(self, node: syn::MethodTurbofish) => {
        if self.settled() {
            return;
        }
        return self.set_help(node, HelpItem::Turbofish);
    }];
    // OMITTED: unreachable from File
    // method![visit_nested_meta(self, node: syn::NestedMeta)];
    method![visit_parenthesized_generic_arguments(
        self,
        node: syn::ParenthesizedGenericArguments
    )];
    method![visit_pat(self, node: syn::Pat)];
    method![@attrs visit_pat_box(self, node: syn::PatBox) {
        token![self, node.box_token, PatBox];
    }];
    method![@attrs visit_pat_ident(self, node: syn::PatIdent) => {
        if self.settled() {
            return;
        }

        if let Some((at_token, subpat)) = &node.subpat {
            if self.between(at_token, subpat) {
                return self.set_help(node, HelpItem::PatIdentSubPat {
                    ident: node.ident.to_string()
                });
            }
        }

        if node.mutability.is_some() && self.has_ancestor::<syn::FnArg>(3) {
            return self.set_help(node, HelpItem::PatIdentMutableArg {
                ident: node.ident.to_string()
            });
        }

        let item = HelpItem::PatIdent {
            mutability: node.mutability.is_some(),
            by_ref: node.by_ref.is_some(),
            ident: node.ident.to_string()
        };
        match (node.by_ref, node.mutability) {
            (Some(by_ref), Some(mutability)) => token![self, by_ref => mutability, * item],
            (Some(by_ref), None) => token![self, by_ref, * item],
            (None, Some(mutability)) => token![self, mutability, * item],
            _ => {}
        }
    }];
    method![@attrs visit_pat_lit(self, node: syn::PatLit)];
    method![@attrs visit_pat_macro(self, node: syn::PatMacro)];
    method![@attrs visit_pat_or(self, node: syn::PatOr) {
        token![self, some node.leading_vert, PatOrLeading];
        for pair in node.cases.pairs() {
            token![self, some pair.punct(), PatOr];
        }
    }];
    method![@attrs visit_pat_path(self, node: syn::PatPath)];
    method![@attrs visit_pat_range(self, node: syn::PatRange) => {
        if self.settled() {
            return;
        }
        return self.set_help(node, HelpItem::PatRange {
            closed: if let syn::RangeLimits::Closed(..) = node.limits {
                true
            } else {
                false
            }
        })
    }];
    method![@attrs visit_pat_reference(self, node: syn::PatReference)];
    // OMITTED: handled in the parent patterns
    // method![visit_pat_rest(self, node: syn::PatRest)];
    method![@attrs visit_pat_slice(self, node: syn::PatSlice)];
    method![@attrs visit_pat_struct(self, node: syn::PatStruct) {
        if node.fields.is_empty() {
            token![self, some node.dot2_token, * HelpItem::PatStruct {
                empty: true,
                bindings: pattern_bindings(&self)
            }];
        }
        token![self, some node.dot2_token, * HelpItem::PatRest {
            of: RestOf::Struct
        }];
    } => {
        if self.settled() {
            return;
        }

        return self.set_help(node, HelpItem::PatStruct {
            empty: false,
            bindings: pattern_bindings(&self)
        });
    }];
    method![@attrs visit_pat_tuple(self, node: syn::PatTuple) => {
        if self.settled() {
            return;
        }

        if self.get_ancestor::<syn::PatTupleStruct>(1).is_some() {
            return;
        }

        if let Some(pat @ syn::Pat::Rest(..)) = node.elems.last() {
            token![self, pat, * HelpItem::PatRest { of: RestOf::Tuple }];
        }

        return self.set_help(node, HelpItem::PatTuple {
            bindings: pattern_bindings(&self)
        });
    }];
    method![@attrs visit_pat_tuple_struct(self, node: syn::PatTupleStruct) => {
        if self.settled() {
            return;
        }

        if let Some(pat @ syn::Pat::Rest(..)) = node.pat.elems.last() {
            token![self, pat, * HelpItem::PatRest { of: RestOf::TupleStruct }];
        }

        return self.set_help(node, HelpItem::PatTupleStruct {
            bindings: pattern_bindings(&self)
        });
    }];
    method![@attrs visit_pat_type(self, node: syn::PatType)];
    method![@attrs visit_pat_wild(self, node: syn::PatWild) @terminal {
        // TODO: special case final, catch-all arm in a match pattern
        return self.set_help(node, HelpItem::PatWild);
    }];
    // TODO:
    // * Fn patterns: Fn(A, B) -> C
    method![visit_path(self, node: syn::Path) {
        let qself = loop {
            let ancestor = if let Some(ancestor) = self.ancestors.last() {
                *ancestor
            } else {
                return;
            };

            if let Some(expr_path) = ancestor.as_any().downcast_ref::<syn::ExprPath>() {
                break &expr_path.qself;
            }
            if let Some(type_path) = ancestor.as_any().downcast_ref::<syn::TypePath>() {
                break &type_path.qself;
            }
            if let Some(pat_path) = ancestor.as_any().downcast_ref::<syn::PatPath>() {
                break &pat_path.qself;
            }

            break &None;
        };

        let simple_qself = qself
            .as_ref()
            .map(|q| q.as_token.is_some())
            .unwrap_or(false);

        if special_path_help(
            self,
            node.leading_colon.filter(|_| simple_qself),
            node.segments.first().map(|s| &s.ident),
            node.segments.len() == 1 && self.has_ancestor::<syn::ExprPath>(1),
        ) {
            return;
        }
    }];
    method![visit_path_arguments(self, node: syn::PathArguments)];
    method![visit_path_segment(self, node: syn::PathSegment) {
        if node.ident == "super" {
            return self.set_help(&node.ident, HelpItem::PathSegmentSuper);
        }
    } => {
        if self.settled() {
            return;
        }
        if let syn::PathArguments::Parenthesized(..) = node.arguments {
            return self.set_help(node, HelpItem::ParenthesizedGenericArguments);
        }
    }];
    method![visit_predicate_eq(self, node: syn::PredicateEq)];
    method![visit_predicate_lifetime(self, node: syn::PredicateLifetime)];
    method![visit_predicate_type(self, node: syn::PredicateType)];
    // OPEN QUESTION: what is the purpose of the `<T>::foo()` syntax?
    // If `T` has an intrinsic `foo()`, both:
    // * T::foo()
    // * <T>::foo()
    // call it, regardless of whether there are additional `foo`s defined in traits.
    // The only thing that I can fathom is that `T::foo()` assumes `T` is a valid
    // path segment, and this is not always true:
    // * <dyn Foo>::foo()
    // * <&Foo>::foo()  ?
    // * <_>::foo()   ?
    fn visit_qself(&mut self, node: &'ast syn::QSelf) {
        if !self.between_spans(node.lt_token.span(), node.gt_token.span()) {
            return;
        }

        self.ancestors.push(node);
        syn::visit::visit_qself(self, node);
        let _ = self.ancestors.pop();

        if self.settled() {
            return;
        }

        return self.set_help_between(
            node.lt_token.span(),
            node.gt_token.span(),
            if node.as_token.is_some() {
                HelpItem::QSelfAsTrait
            } else {
                HelpItem::QSelf
            },
        );
    }
    // OMITTED: handled in parents
    // fn visit_range_limits(&mut self, node: &'ast syn::RangeLimits) {
    method![@attrs visit_receiver(self, node: syn::Receiver) @terminal {
        let item = match (&node.reference, &node.mutability) {
            (Some(_), Some(_)) => HelpItem::MutSelf { explicit: false, mutability: false },
            (Some(_), None) => HelpItem::RefSelf { explicit: false, mutability: false },
            (None, mutability) => {
                HelpItem::ValueSelf {
                    explicit: false,
                    mutability: mutability.is_some()
                }
            }
        };
        return self.set_help(&node, item);
    }];
    method![visit_return_type(self, node: syn::ReturnType) {
        let rarrow = if let syn::ReturnType::Type(rarrow, _) = node {
            Some(rarrow)
        } else {
            None
        };

        token![self, some rarrow, * HelpItem::RArrow {
            return_of: if self.get_ancestor::<syn::TypeBareFn>(1).is_some() {
                ReturnOf::BareFunctionType
            } else if self.get_ancestor::<syn::ExprClosure>(1).is_some() {
                ReturnOf::Closure
            } else if self.get_ancestor::<syn::ParenthesizedGenericArguments>(1).is_some() {
                ReturnOf::FnTrait
            } else {
                // TODO: distinguish methods
                ReturnOf::Function
            }
        }];
    }];
    method![visit_signature(self, node: syn::Signature) {
        token![self, some node.asyncness, AsyncFn];
        token![self, some node.constness, ConstFn];
        token![self, some node.abi, FnAbi];
        token![self, some node.unsafety, UnsafeFn];
    }];
    method![visit_stmt(self, node: syn::Stmt)];
    method![visit_trait_bound(self, node: syn::TraitBound) {
        if let Some((bound_lifetimes, lifetime, multiple)) = node.lifetimes
            .as_ref()
            .filter(|bound_lifetimes| self.within(bound_lifetimes))
            .and_then(|bound_lifetimes| {
                bound_lifetimes.lifetimes.first().map(|lt| (bound_lifetimes, lt, bound_lifetimes.lifetimes.len() > 1))
            }) {
                return self.set_help(bound_lifetimes, HelpItem::BoundLifetimesTraitBound {
                    lifetime: format!("{}", lifetime.lifetime),
                    multiple,
                    ty: format!("{}", node.path.to_token_stream())
                });
            }
    } => {
        if self.settled() {
            return;
        }
        if let syn::TraitBoundModifier::Maybe(..) = node.modifier {
            return self.set_help(node, HelpItem::TraitBoundModifierQuestion {
                sized: node.path.is_ident("Sized")
            });
        }
    }];
    // OMITTED: `visit_trait_bound` catches this
    // method![visit_trait_bound_modifier(self, node: syn::TraitBoundModifier )];
    method![visit_trait_item(self, node: syn::TraitItem)];
    method![@attrs visit_trait_item_const(self, node: syn::TraitItemConst) {
        token![self, node.const_token, TraitItemConst];
    }];
    method![@attrs visit_trait_item_macro(self, node: syn::TraitItemMacro)];
    method![@attrs visit_trait_item_method(self, node: syn::TraitItemMethod) {
        token![self, node.sig.fn_token, * HelpItem::FnToken { of: FnOf::TraitMethod, name: node.sig.ident.to_string() }];
        token![self, node.sig.ident, TraitItemMethod];
    }];
    method![@attrs visit_trait_item_type(self, node: syn::TraitItemType) {
        token![self, node.type_token, TraitItemType];
    }];
    method![visit_type(self, node: syn::Type)];
    method![visit_type_array(self, node: syn::TypeArray) {
        if self.between_locations(node.span().start(), node.elem.span().start()) ||
            self.between_spans(node.semi_token.span(), node.span()) {
            return self.set_help(node, HelpItem::TypeArray);
        }
    }];
    method![visit_type_bare_fn(self, node: syn::TypeBareFn) {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        token![self, some node.lifetimes, BoundLifetimesBareFnType];
    } => {
        if !self.settled() {
            return self.set_help(node, HelpItem::TypeBareFn);
        }
    }];
    method![visit_type_group(self, node: syn::TypeGroup)];
    method![visit_type_impl_trait(self, node: syn::TypeImplTrait) {
        token![self, node.impl_token, TypeImplTrait];
    }];
    method![visit_type_infer(self, node: syn::TypeInfer) @terminal {
        return self.set_help(node, HelpItem::TypeInfer);
    }];
    method![visit_type_macro(self, node: syn::TypeMacro)];
    method![visit_type_never(self, node: syn::TypeNever) @terminal {
        return self.set_help(node, HelpItem::TypeNever);
    }];
    method![@attrs visit_type_param(self, node: syn::TypeParam)];
    method![visit_type_param_bound(self, node: syn::TypeParamBound)];
    method![visit_type_paren(self, node: syn::TypeParen)];
    method![visit_type_path(self, node: syn::TypePath) => {
        if self.settled() {
            return;
        }
        if let Some(item) = well_known_type(node) {
            return self.set_help(node, item);
        }
    }];
    method![visit_type_ptr(self, node: syn::TypePtr) {
        let end = node.const_token
            .map(|t| t.span())
            .or_else(|| node.mutability.map(|t| t.span()));

        if let Some(end) = end {
            return self.set_help_between(node.span(), end, if node.const_token.is_some() {
                HelpItem::TypeConstPtr
            } else {
                HelpItem::TypeMutPtr
            });
        }
    }];
    method![visit_type_reference(self, node: syn::TypeReference) {
        if let syn::Type::Path(type_path) = &*node.elem {
            if let Some(HelpItem::KnownTypeStr) = well_known_type(type_path) {
                return self.set_help(node, HelpItem::KnownTypeStrSlice {
                    mutability: node.mutability.is_some()
                });
            }
        }
    } => {
        let last_span = node.mutability.map(|t| t.span())
            .or_else(|| node.lifetime.as_ref().map(|t| t.span()))
            .unwrap_or_else(|| node.and_token.span());

        if self.between_spans(node.and_token.span(), last_span) {
            return self.set_help(&node, HelpItem::TypeReference {
                lifetime: node.lifetime.is_some(),
                mutable: node.mutability.is_some(),
                ty: format!("{}", node.elem.to_token_stream())
            });
        }
    }];
    method![visit_type_slice(self, node: syn::TypeSlice) => {
        if !self.settled() {
            return self.set_help(node, HelpItem::TypeSlice {
                dynamic: self.get_ancestor::<syn::TypeReference>(2).is_some(),
                ty: node.elem.to_token_stream().to_string()
            });
        }
    }];
    // Fun fact: a legacy trait object without `dyn` can probably only be recognized
    // by compiling the code.
    method![visit_type_trait_object(self, node: syn::TypeTraitObject) {
        if let Some(plus_token) = node.bounds.pairs()
            .filter_map(|pair| pair.punct().cloned())
            .find(|punct| self.within(punct))
        {
            return self.set_help(&plus_token, HelpItem::TypeParamBoundAdd);
        }
    } => {
        if self.settled() {
            return;
        }

        let ty = if let Some(syn::TypeParamBound::Trait(trait_bound)) = node.bounds.first() {
            trait_bound
        } else {
            return;
        };

        let lifetime = node.bounds.iter().filter_map(|bound| match bound {
            syn::TypeParamBound::Lifetime(lifetime) => Some(lifetime),
            _ => None
        })
            .next();

        let multiple = node.bounds.iter().filter(|bound| match bound {
            syn::TypeParamBound::Trait(..) => true,
            _ => false
        })
            .skip(1)
            .next()
            .is_some();

        return self.set_help(node, HelpItem::TypeTraitObject {
            lifetime: lifetime.map(|lt| format!("{}", lt)),
            multiple,
            dyn_: node.dyn_token.is_some(),
            ty: format!("{}", ty.path.to_token_stream())
        });
    }];
    method![visit_type_tuple(self, node: syn::TypeTuple) {
        if node.elems.is_empty() {
            return self.set_help(node, HelpItem::TypeTupleUnit);
        }
    } => {
        if !self.settled() {
            return self.set_help(node, HelpItem::TypeTuple);
        }
    }];
    method![visit_un_op(self, node: syn::UnOp)];
    method![visit_use_glob(self, node: syn::UseGlob) @terminal {
        return self.set_help(node, HelpItem::UseGlob);
    }];
    method![visit_use_group(self, node: syn::UseGroup) => {
        if self.settled() {
            return;
        }

        if let Some(path) = self.get_ancestor::<syn::UsePath>(2) {
            return self.set_help(node, HelpItem::UseGroup {
                parent: path.ident.to_string()
            });
        }
    }];
    method![visit_use_name(self, node: syn::UseName) @terminal {
        if node.ident != "self" {
            return;
        }

        if self.get_ancestor::<syn::UseGroup>(2).is_none() {
            return;
        }

        let path_ancestor = if let Some(path) = self.get_ancestor::<syn::UsePath>(4) {
            path
        } else {
            return;
        };

        return self.set_help(node, HelpItem::UseGroupSelf {
            parent: path_ancestor.ident.to_string()
        });
    }];
    method![visit_use_path(self, node: syn::UsePath) => {
        let mut root_path = true;

        for i in (2..).step_by(2) {
            if self.get_ancestor::<syn::UseGroup>(i).is_some() {
                continue;
            }

            root_path = self.get_ancestor::<syn::ItemUse>(i).is_some();
            break;
        }

        if root_path && special_path_help(self, None, Some(&node.ident), false) {
            return;
        }

        if node.ident == "super" {
            return self.set_help(&node.ident, HelpItem::PathSegmentSuper);
        }
    }];
    method![visit_use_rename(self, node: syn::UseRename) {
        token![self, node.as_token, AsRename];
    }];
    method![visit_use_tree(self, node: syn::UseTree)];
    method![@attrs visit_variadic(self, node: syn::Variadic)];
    method![@attrs visit_variant(self, node: syn::Variant) => {
        if self.settled() {
            return;
        }
        if let Some((eq_token, discriminant)) = &node.discriminant {
            if self.between(&eq_token, &discriminant) {
                return self.set_help_between(eq_token.span(), discriminant.span(), HelpItem::VariantDiscriminant {
                    name: node.ident.to_string()
                });
            }
        }
        let name = if let Some(item_enum) = self.get_ancestor::<syn::ItemEnum>(1) {
            item_enum.ident.to_string()
        } else {
            return;
        };

        return self.set_help(node, HelpItem::Variant {
            name,
            fields: match node.fields {
                syn::Fields::Named(..) => Some(Fields::Named),
                syn::Fields::Unnamed(..) => Some(Fields::Unnamed),
                syn::Fields::Unit => None
            }
        });
    }];
    method![visit_vis_crate(self, node: syn::VisCrate) @terminal {
        return self.set_help(node, HelpItem::VisCrate);
    }];
    method![visit_vis_public(self, node: syn::VisPublic) @terminal {
        return self.set_help(node, HelpItem::VisPublic);
    }];
    method![visit_vis_restricted(self, node: syn::VisRestricted) @terminal {
        let path = match &*node.path {
            path if path.is_ident("self") => VisRestrictedPath::Self_,
            path if path.is_ident("super") => VisRestrictedPath::Super,
            path if path.is_ident("crate") => VisRestrictedPath::Crate,
            _ => VisRestrictedPath::Path
        };
        return self.set_help(node, HelpItem::VisRestricted {
            path,
            in_: node.in_token.is_some()
        });
    }];
    method![visit_visibility(self, node: syn::Visibility)];
    method![visit_where_clause(self, node: syn::WhereClause) {
        token![self, node.where_token, WhereClause];
    }];
    method![visit_where_predicate(self, node: syn::WherePredicate)];
}

fn vis_span(vis: &syn::Visibility) -> Option<Span> {
    if let syn::Visibility::Inherited = vis {
        None
    } else {
        Some(vis.span())
    }
}

fn outer_attr(attr: &syn::Attribute) -> bool {
    match attr.style {
        syn::AttrStyle::Outer => true,
        syn::AttrStyle::Inner(..) => false,
    }
}

fn pattern_bindings(visitor: &IntersectionVisitor) -> Option<BindingOf> {
    if visitor.has_ancestor::<syn::Local>(2)
        || (visitor.has_ancestor::<syn::Local>(4) && visitor.has_ancestor::<syn::PatType>(2))
    {
        return Some(BindingOf::Let);
    }
    if visitor.has_ancestor::<syn::FnArg>(4) {
        return Some(BindingOf::Arg);
    }

    None
}

fn special_path_help(
    visitor: &mut IntersectionVisitor,
    leading_colon: Option<syn::token::Colon2>,
    leading_segment: Option<&syn::Ident>,
    can_be_receiver: bool,
) -> bool {
    if let Some(leading_colon) = leading_colon {
        if visitor.within(leading_colon) {
            visitor.set_help(&leading_colon, HelpItem::PathLeadingColon);
            return true;
        }
    }

    let mut settled = false;
    if let Some(ident) = leading_segment {
        if visitor.within(&ident) {
            if ident == "super" {
                visitor.set_help(&ident, HelpItem::PathSegmentSuper);
                settled = true;
            } else if ident == "self" {
                visitor.set_help(
                    &ident,
                    if can_be_receiver {
                        HelpItem::ReceiverPath {
                            method: visitor
                                .ancestors
                                .iter()
                                .rev()
                                .map(|ancestor| (*ancestor).as_any())
                                .filter_map(|any| {
                                    any.downcast_ref::<syn::ImplItemMethod>()
                                        .map(|method| &method.sig)
                                        .or_else(|| {
                                            any.downcast_ref::<syn::TraitItemMethod>()
                                                .map(|method| &method.sig)
                                        })
                                })
                                .next()
                                .map(|sig| sig.ident.to_string()),
                        }
                    } else {
                        HelpItem::PathSegmentSelf
                    },
                );
                settled = true;
            } else if ident == "Self" {
                visitor.set_help(&ident, HelpItem::PathSegmentSelfType);
                settled = true;
            } else if ident == "crate" {
                visitor.set_help(&ident, HelpItem::PathSegmentCrate);
                settled = true;
            }
        }
    }

    settled
}

pub fn within_locations(loc: LineColumn, start: LineColumn, end: LineColumn) -> bool {
    (start.line < loc.line || (start.line == loc.line && start.column <= loc.column))
        && (loc.line < end.line || (loc.line == end.line && loc.column <= end.column))
}

pub fn well_known_type(type_path: &syn::TypePath) -> Option<HelpItem> {
    let path = &type_path.path;
    if type_path.qself.is_some() || path.leading_colon.is_some() || path.segments.len() > 1 {
        return None;
    }

    let ident = match path.segments.first() {
        Some(segment) if segment.arguments.is_empty() => &segment.ident,
        _ => return None,
    };

    if ident == "u8" {
        Some(HelpItem::KnownTypeU8)
    } else if ident == "u16" {
        Some(HelpItem::KnownTypeU16)
    } else if ident == "u32" {
        Some(HelpItem::KnownTypeU32)
    } else if ident == "u64" {
        Some(HelpItem::KnownTypeU64)
    } else if ident == "u128" {
        Some(HelpItem::KnownTypeU128)
    } else if ident == "usize" {
        Some(HelpItem::KnownTypeUSize)
    } else if ident == "i8" {
        Some(HelpItem::KnownTypeI8)
    } else if ident == "i16" {
        Some(HelpItem::KnownTypeI16)
    } else if ident == "i32" {
        Some(HelpItem::KnownTypeI32)
    } else if ident == "i64" {
        Some(HelpItem::KnownTypeI64)
    } else if ident == "i128" {
        Some(HelpItem::KnownTypeI128)
    } else if ident == "isize" {
        Some(HelpItem::KnownTypeISize)
    } else if ident == "char" {
        Some(HelpItem::KnownTypeChar)
    } else if ident == "bool" {
        Some(HelpItem::KnownTypeBool)
    } else if ident == "f32" {
        Some(HelpItem::KnownTypeF32)
    } else if ident == "f64" {
        Some(HelpItem::KnownTypeF64)
    } else if ident == "str" {
        Some(HelpItem::KnownTypeStr)
    } else {
        None
    }
}

fn raw_string_literal(mut literal: String, prefix: &str) -> Option<String> {
    if literal.starts_with(prefix) {
        for _ in 0..prefix.len() {
            literal.remove(0);
        }
        let open = literal
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '"')
            .next();

        open.map(|(idx, _)| {
            literal.truncate(idx);
            literal
        })
    } else {
        None
    }
}
