use proc_macro2::{LineColumn, Span};
use syn::spanned::Spanned;
use syn::visit::Visit;

std::include!(concat!(env!("OUT_DIR"), "/help.rs"));

#[cfg(feature = "dev")]
use super::log;

use super::RustCode;

#[derive(Debug, Clone, Copy)]
pub enum HelpItem {
    Use,
    Macro,
    Attribute { outer: bool },
    ItemExternCrate,
    ItemInlineMod,
    ItemExternMod,
    Unknown,
    FnToken,
    TraitItemMethod,
    ImplItemMethod,
    AsRename,
    QualifiedAs,
    AsRenameExternCrate,
    AsCast,
    ExprClosureArguments,
    ExprClosureAsync,
    ExprClosureStatic,
    AsyncExpression,
    AsyncFn,
    AwaitExpression,
    Break { label: bool, expr: bool },
    ImplItemConst,
    TraitItemConst,
    ItemConst,
    ConstPtr,
    MutPtr,
    ConstParam,
    ConstFn,
    ExprContinue { label: bool },
    VisPublic,
    VisCrate,
    VisRestricted,
    Dyn,
    Else,
    ItemEnum,
    ItemForeignModAbi,
    FnAbi,
    TypeBareFnAbi,
    True,
    False,
    FnTypeToken,
    ItemImplForTrait,
    ExprForLoopToken,
    BoundLifetimes,
    IfLet,
    If,
    ItemImpl,
    TypeImplTrait,
    WhileLet,
    While,
    Local,
    LocalMut,
    Label,
    ExprLoopToken,
    ExprMatchToken,
    ArmIfGuard,
    Move,
    FnArgMut,
    MutSelf,
    ValueSelf,
    RefSelf,
    PatIdentRef,
    PatIdentMut,
    PatIdentRefMut,
    StaticMut,
    Static,
    TypeReference { mutable: bool, lifetime: bool },
    ExprReference { mutable: bool },
    ExprReturn,
    PathSegmentSelf,
    ExprUnsafe,
    ForeignItemType,
    ImplItemType,
    ItemUnsafeImpl,
    ItemStruct,
    ItemUnsafeTrait,
    ItemTrait,
    ItemType,
    ItemUnion,
    PathSegmentSuper,
    UnsafeFn,
    TraitItemType,
    WhereClause,
    TypeBareUnsafeFn,
    ItemTraitAlias,
    Field { named: bool },
    FatArrow,
    DocBlock { outer: bool },
    RArrow,
    ExprRangeHalfOpen { from: bool, to: bool },
    ExprRangeClosed { from: bool, to: bool },
    StaticLifetime,
    TraitBound,
}

impl HelpItem {
    pub fn message(&self) -> &'static str {
        help_to_message(*self).0
    }

    pub fn title(&self) -> &'static str {
        help_to_message(*self).1
    }

    pub fn keyword(&self) -> Option<&'static str> {
        help_to_message(*self).3
    }

    pub fn book(&self) -> Option<&'static str> {
        help_to_message(*self).2
    }
}

pub(super) struct IntersectionVisitor<'ast> {
    location: LineColumn,
    help: HelpItem,
    item_location: (LineColumn, LineColumn),
    ancestors: Vec<Ancestor<'ast>>,
    next_ancestor: Ancestor<'ast>,
    #[cfg(feature = "dev")]
    debug: String,
}

#[cfg_attr(feature = "dev", derive(Debug))]
struct Ancestor<'a> {
    span: Span,
    attributes: &'a [syn::Attribute],
    #[cfg(feature = "dev")]
    type_: &'static str,
}

impl<'a> Ancestor<'a> {
    fn new() -> Self {
        Ancestor {
            span: Span::call_site(),
            attributes: &[],
            #[cfg(feature = "dev")]
            type_: "",
        }
    }
}

pub struct VisitorResult {
    pub help: HelpItem,
    pub item_location: (LineColumn, LineColumn),
    #[cfg(feature = "dev")]
    pub debug: String,
}

impl<'ast> IntersectionVisitor<'ast> {
    pub fn new(location: LineColumn) -> Self {
        #[cfg(feature = "dev")]
        {
            Self {
                location,
                item_location: (location, location),
                help: HelpItem::Unknown,
                ancestors: vec![],
                next_ancestor: Ancestor::new(),
                debug: "".to_string(),
            }
        }
        #[cfg(not(feature = "dev"))]
        {
            Self {
                location,
                item_location: (location, location),
                help: HelpItem::Unknown,
                ancestors: vec![],
                next_ancestor: Ancestor::new(),
            }
        }
    }

    pub fn visit(mut self, code: &'ast RustCode) -> VisitorResult {
        match code {
            RustCode::File(file) => self.visit_file(file),
            RustCode::Block(block) => self.visit_block(block),
        };

        VisitorResult {
            help: self.help,
            item_location: self.item_location,
            #[cfg(feature = "dev")]
            debug: self.debug,
        }
    }

    fn within<S: Spanned>(&self, item: S) -> bool {
        let span = item.span();
        self.within_spans(span.start(), span.end())
    }

    fn between<S: Spanned + ?Sized, T: Spanned + ?Sized>(&self, start: &S, end: &T) -> bool {
        self.within_spans(start.span().start(), end.span().end())
    }

    fn within_spans(&self, start: LineColumn, end: LineColumn) -> bool {
        let loc = self.location;
        super::within_spans(loc, start, end)
    }

    fn set_help<S: Spanned + ?Sized>(&mut self, item: &S, help: HelpItem) {
        self.set_help_between(item.span(), item.span(), help);
    }

    fn set_help_span(&mut self, span: Span, help: HelpItem) {
        self.set_help_between(span, span, help);
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
}

macro_rules! method {
    (@_spancheck, $self:ident, $node:ident, $ty:path) => {{
        if !$self.within(&$node) {
            return;
        }
        $self.next_ancestor.span = $node.span();
        #[cfg(feauture = "dev")]
        {
            $self.next_ancestor.type_ = stringify!($ty);
        }
    }};
    (@_nospancheck, $self:ident, $node:ident, $ty:path) => {};
    (@_debug, $name:ident, $self:ident, $node:ident) => {{
        #[cfg(feature = "dev")]
        {
            log(stringify!($name));
        }
    }};
    (@_attrs, $self:ident, $node:ident, $body:expr) => {{
        let mut within_attribute = false;

        for attr in $node.attrs.iter() {
            if $self.within(attr) {
                within_attribute = true;
                break;
            }
        }

        $self.next_ancestor.attributes = &$node.attrs[..];

        if !within_attribute {
            $body
        }
    }};
    (@_ancestor @_spancheck, $self:ident) => {
        let mut next_ancestor = Ancestor::new();
        std::mem::swap(&mut $self.next_ancestor, &mut next_ancestor);
        $self.ancestors.push(next_ancestor);
    };
    (@_ancestor @_nospancheck, $self:ident) => {};
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body $body:expr, @ $spancheck:ident) => {
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@_debug, $name, $self, $node];
            method![@$spancheck, $self, $node, $ty];
            $body;
            method![@_ancestor @$spancheck, $self];
            syn::visit::$name($self, $node);
        }
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body @terminal $body:expr, @ $spancheck:ident) => {
        #[allow(unreachable_code)]
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@$spancheck, $self, $node, $ty];
            method![@_debug, $name, $self, $node];
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
    ($name:ident($self:ident, $node:ident: $ty:path) $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body $body, @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, $body], @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path)) => {
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, { () }], @_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal $body, @_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path)) => {
        method![@_impl, $name, $self, $node, $ty, @_body { () }, @_spancheck];
    };
    (@nospancheck $name:ident($self:ident, $node:ident: $ty:path)) => {
        method![@_impl, $name, $self, $node, $ty, @_body { () }, @_nospancheck];
    };
}

macro_rules! token {
    ($self:expr, $token:expr, $item:ident) => {
        token![$self, $token, *HelpItem::$item];
    };
    ($self:expr, $token:expr, * $item:expr) => {
        if $self.within(&$token) {
            return $self.set_help(&$token, $item);
        }
    };
    ($self:expr, some $token:expr, $item:ident) => {
        if let Some(ref inner) = &$token {
            token![$self, inner, $item];
        }
    };
}

// TODO
// - asterisk in glob imports
// - => in match arms
// - operators: question mark, <<, >>, +, -, /, %, +=, -=, *=, /=, %=, ||, &&, range operators, [..]
// - distinguish macro_rules!
impl<'ast> Visit<'ast> for IntersectionVisitor<'ast> {
    method![visit_abi(self, node: syn::Abi)];
    method![visit_angle_bracketed_generic_arguments(
        self,
        node: syn::AngleBracketedGenericArguments
    )];
    method![visit_arm(self, node: syn::Arm) {
        token![self, node.fat_arrow_token, FatArrow];
        if let Some((if_token, _)) = node.guard {
            token![self, if_token, ArmIfGuard];
        }
    }];
    // method![visit_attr_style(self, node: syn::AttrStyle)];
    method![visit_attribute(self, node: syn::Attribute) @terminal {
        let outer = outer_attr(node);
        if node.path.is_ident("doc") {
            let bounds = if let Some(Ancestor { attributes, .. }) = self.ancestors.last() {
                let this_idx = attributes.iter()
                    .enumerate()
                    .find(|(_, attr)| std::ptr::eq(node, *attr))
                    .expect("attr in list")
                    .0;

                let last = attributes.iter()
                    .enumerate()
                    .filter(|&(i, attr)| i >= this_idx && attr.path.is_ident("doc") && outer_attr(attr) == outer)
                    .last()
                    .expect("last")
                    .1;

                let start = attributes.iter()
                    .enumerate()
                    .rev()
                    .filter(|&(i, attr)| i <= this_idx && !(attr.path.is_ident("doc") && outer_attr(attr) == outer))
                    .next()
                    .map(|(i, _)| &attributes[i - 1])
                    .unwrap_or(&attributes[0]);

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
    method![visit_bare_fn_arg(self, node: syn::BareFnArg)];
    method![visit_bin_op(self, node: syn::BinOp)];
    method![visit_binding(self, node: syn::Binding)];
    method![visit_block(self, node: syn::Block)];
    method![visit_bound_lifetimes(self, node: syn::BoundLifetimes) @terminal {
        return self.set_help(&node, HelpItem::BoundLifetimes);
    }];
    method![visit_const_param(self, node: syn::ConstParam) {
        token![self, node.const_token, ConstParam];
    }];
    method![visit_constraint(self, node: syn::Constraint)];
    // method![visit_data(self, node: syn::Data)];
    // method![visit_data_enum(self, node: syn::DataEnum)];
    // method![visit_data_struct(self, node: syn::DataStruct)];
    // method![visit_data_union(self, node: syn::DataUnion)];
    fn visit_derive_input(&mut self, _: &'ast syn::DeriveInput) {}
    method![visit_expr(self, node: syn::Expr)];
    method![visit_expr_array(self, node: syn::ExprArray)];
    method![visit_expr_assign(self, node: syn::ExprAssign)];
    method![visit_expr_assign_op(self, node: syn::ExprAssignOp)];
    method![visit_expr_async(self, node: syn::ExprAsync) {
        token![self, node.async_token, AsyncExpression];
        if let Some(capture) = node.capture {
            token![self, capture, Move];
        }
    }];
    method![visit_expr_await(self, node: syn::ExprAwait) {
        token![self, node.await_token, AwaitExpression];
    }];
    method![visit_expr_binary(self, node: syn::ExprBinary)];
    method![visit_expr_block(self, node: syn::ExprBlock)];
    method![visit_expr_box(self, node: syn::ExprBox)];
    method![visit_expr_break(self, node: syn::ExprBreak) {
        if self.within(node.break_token) {
            return self.set_help(&node, HelpItem::Break { expr: node.expr.is_some(), label: node.label.is_some() });
        }
    }];
    method![visit_expr_call(self, node: syn::ExprCall)];
    method![visit_expr_cast(self, node: syn::ExprCast) {
        token![self, node.as_token, AsCast];
    }];
    method![visit_expr_closure(self, node: syn::ExprClosure) {
        token![self, node.or1_token, ExprClosureArguments];
        token![self, node.or2_token, ExprClosureArguments];
        token![self, some node.asyncness, ExprClosureAsync];
        token![self, some node.capture, Move];
        token![self, some node.movability, ExprClosureStatic];
    }];
    method![visit_expr_continue(self, node: syn::ExprContinue) {
        self.set_help(node, HelpItem::ExprContinue { label: node.label.is_some() });
    }];
    method![visit_expr_field(self, node: syn::ExprField)];
    method![visit_expr_for_loop(self, node: syn::ExprForLoop) {
        token![self, node.for_token, ExprForLoopToken];
        token![self, node.in_token, ExprForLoopToken];
    }];
    method![visit_expr_group(self, node: syn::ExprGroup)];
    method![visit_expr_if(self, node: syn::ExprIf) {
        if let syn::Expr::Let(syn::ExprLet { let_token, .. }) = *node.cond {
            if self.within_spans(node.if_token.span().start(), let_token.span().end()) {
                return self.set_help_between(node.if_token.span(), let_token.span(), HelpItem::IfLet);
            }
        } else {
            token![self, node.if_token, If];
        };
        if let Some((else_token, _)) = node.else_branch {
            token![self, else_token, Else];
        }
    }];
    method![visit_expr_index(self, node: syn::ExprIndex)];
    method![visit_expr_let(self, node: syn::ExprLet)];
    method![visit_expr_lit(self, node: syn::ExprLit)];
    method![visit_expr_loop(self, node: syn::ExprLoop) {
        token![self, node.loop_token, ExprLoopToken];
    }];
    method![visit_expr_macro(self, node: syn::ExprMacro)];
    method![visit_expr_match(self, node: syn::ExprMatch) {
        token![self, node.match_token, ExprMatchToken];
    }];
    method![visit_expr_method_call(self, node: syn::ExprMethodCall)];
    method![visit_expr_paren(self, node: syn::ExprParen)];
    method![visit_expr_path(self, node: syn::ExprPath)];
    method![visit_expr_range(self, node: syn::ExprRange) @terminal {
        let from = node.from.is_some();
        let to = node.to.is_some();
        match node.limits {
            syn::RangeLimits::HalfOpen(..) => return self.set_help(node, HelpItem::ExprRangeHalfOpen { from, to }),
            syn::RangeLimits::Closed(..) => return self.set_help(node, HelpItem::ExprRangeClosed { from, to }),
        }
    }];
    method![visit_expr_reference(self, node: syn::ExprReference) {
        let last_span = node.mutability.map(|t| t.span())
            .unwrap_or_else(|| node.and_token.span());

        if self.within_spans(node.and_token.span().start(), last_span.end()) {
            return self.set_help(&node, HelpItem::ExprReference {
                mutable: node.mutability.is_some()
            });
        }
    }];
    method![visit_expr_repeat(self, node: syn::ExprRepeat)];
    method![visit_expr_return(self, node: syn::ExprReturn) {
        token![self, node.return_token, ExprReturn];
    }];
    method![visit_expr_struct(self, node: syn::ExprStruct)];
    method![visit_expr_try(self, node: syn::ExprTry)];
    method![visit_expr_try_block(self, node: syn::ExprTryBlock)];
    method![visit_expr_tuple(self, node: syn::ExprTuple)];
    method![visit_expr_type(self, node: syn::ExprType)];
    method![visit_expr_unary(self, node: syn::ExprUnary)];
    method![visit_expr_unsafe(self, node: syn::ExprUnsafe) {
        token![self, node.unsafe_token, ExprUnsafe];
    }];
    method![visit_expr_while(self, node: syn::ExprWhile) {
        let while_token = if let syn::Expr::Let(..) = *node.cond {
            HelpItem::WhileLet
        } else {
            HelpItem::While
        };
        token![self, node.while_token, * while_token];
    }];
    method![visit_expr_yield(self, node: syn::ExprYield)];
    method![@attrs visit_field(self, node: syn::Field) {
        // TODO: attrs + terminal
        if self.within(node) {
            return self.set_help(node, HelpItem::Field { named: node.ident.is_some() });
        }
    }];
    method![visit_field_pat(self, node: syn::FieldPat)];
    method![visit_field_value(self, node: syn::FieldValue)];
    method![visit_fields(self, node: syn::Fields)];
    method![visit_fields_named(self, node: syn::FieldsNamed)];
    method![visit_fields_unnamed(self, node: syn::FieldsUnnamed)];
    method![@attrs visit_file(self, node: syn::File)];
    method![visit_fn_arg(self, node: syn::FnArg)];
    method![visit_foreign_item(self, node: syn::ForeignItem)];
    method![visit_foreign_item_fn(self, node: syn::ForeignItemFn)];
    method![visit_foreign_item_macro(self, node: syn::ForeignItemMacro)];
    method![visit_foreign_item_static(
        self,
        node: syn::ForeignItemStatic
    ) {
        let end = node.mutability.as_ref().map(Spanned::span)
            .unwrap_or_else(|| node.static_token.span());

        if self.within_spans(node.static_token.span().start(), end.end()) {
            return self.set_help_between(node.static_token.span(), end, if node.mutability.is_some() {
                HelpItem::StaticMut
            } else {
                HelpItem::Static
            });
        }
    }];
    method![visit_foreign_item_type(self, node: syn::ForeignItemType) {
        token![self, node.type_token, ForeignItemType];
    }];
    method![visit_generic_argument(self, node: syn::GenericArgument)];
    method![visit_generic_method_argument(
        self,
        node: syn::GenericMethodArgument
    )];
    method![visit_generic_param(self, node: syn::GenericParam)];
    method![@nospancheck visit_generics(self, node: syn::Generics)];
    // method![visit_ident(self, node: syn::Ident)];
    method![visit_impl_item(self, node: syn::ImplItem)];
    method![visit_impl_item_const(self, node: syn::ImplItemConst) {
        token![self, node.const_token, ImplItemConst];
    }];
    method![visit_impl_item_macro(self, node: syn::ImplItemMacro)];
    method![@attrs visit_impl_item_method(self, node: syn::ImplItemMethod) {
        token![self, node.sig.ident, ImplItemMethod];
        token![self, node.sig.fn_token, FnToken];
    }];
    method![visit_impl_item_type(self, node: syn::ImplItemType) {
        token![self, node.type_token, ImplItemType];
    }];
    method![visit_index(self, node: syn::Index)];
    method![visit_item(self, node: syn::Item)];
    method![visit_item_const(self, node: syn::ItemConst) {
        token![self, node.const_token, ItemConst];
    }];
    method![visit_item_enum(self, node: syn::ItemEnum) {
        token![self, node.enum_token, ItemEnum];
    }];
    method![@attrs visit_item_extern_crate(self, node: syn::ItemExternCrate) {
        if let Some((as_token, _)) = node.rename {
            token![self, as_token, AsRenameExternCrate];
        }
        let start = vis_span(&node.vis).unwrap_or_else(|| node.extern_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemExternCrate);
    }];
    method![visit_item_fn(self, node: syn::ItemFn) {
        token![self, node.sig.fn_token, FnToken];
    }];
    method![visit_item_foreign_mod(self, node: syn::ItemForeignMod) {
        token![self, node.abi, ItemForeignModAbi];
    }];
    method![visit_item_impl(self, node: syn::ItemImpl) {
        token![self, some node.unsafety, ItemUnsafeImpl];
        if let Some((_, _, for_token)) = node.trait_ {
            token![self, for_token, ItemImplForTrait];
        }
        token![self, node.impl_token, ItemImpl];
    }];
    method![visit_item_macro(self, node: syn::ItemMacro)];
    method![visit_item_macro2(self, node: syn::ItemMacro2)];
    method![@attrs visit_item_mod(self, node: syn::ItemMod) {
        if !self.within(&node.vis) {
            if let Some(..) = node.content {
                if self.between(&node.mod_token, &node.ident) {
                    return self.set_help_between(node.mod_token.span(), node.ident.span(), HelpItem::ItemInlineMod);
                }
            } else {
                return self.set_help(&node, HelpItem::ItemExternMod);
            }
        }
    }];
    method![visit_item_static(self, node: syn::ItemStatic) {
        let end = node.mutability.as_ref().map(Spanned::span)
            .unwrap_or_else(|| node.static_token.span());

        if self.within_spans(node.static_token.span().start(), end.end()) {
            return self.set_help_between(node.static_token.span(), end, if node.mutability.is_some() {
                HelpItem::StaticMut
            } else {
                HelpItem::Static
            });
        }
    }];
    method![visit_item_struct(self, node: syn::ItemStruct) {
        token![self, node.struct_token, ItemStruct];
    }];
    method![visit_item_trait(self, node: syn::ItemTrait) {
        token![self, some node.unsafety, ItemUnsafeTrait];
        token![self, node.trait_token, ItemTrait];
    }];
    method![visit_item_trait_alias(self, node: syn::ItemTraitAlias) {
        token![self, node.trait_token, ItemTraitAlias];
    }];
    method![visit_item_type(self, node: syn::ItemType) {
        token![self, node.type_token, ItemType];
    }];
    method![visit_item_union(self, node: syn::ItemUnion) {
        token![self, node.union_token, ItemUnion];
    }];
    method![@attrs visit_item_use(self, node: syn::ItemUse) {
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::Use);
    }];
    method![visit_label(self, node: syn::Label) @terminal {
        return self.set_help(&node, HelpItem::Label);
    }];
    method![visit_lifetime(self, node: syn::Lifetime) {
        if node.ident == "static" {
            return self.set_help(node, HelpItem::StaticLifetime);
        }
    }];
    method![visit_lifetime_def(self, node: syn::LifetimeDef)];
    method![visit_lit(self, node: syn::Lit)];
    method![visit_lit_bool(self, node: syn::LitBool) @terminal {
        return self.set_help(node, if node.value { HelpItem::True } else { HelpItem:: False });
    }];
    method![visit_lit_byte(self, node: syn::LitByte)];
    method![visit_lit_byte_str(self, node: syn::LitByteStr)];
    method![visit_lit_char(self, node: syn::LitChar)];
    method![visit_lit_float(self, node: syn::LitFloat)];
    method![visit_lit_int(self, node: syn::LitInt)];
    method![visit_lit_str(self, node: syn::LitStr)];
    method![visit_local(self, node: syn::Local) {
        if let syn::Pat::Ident(syn::PatIdent { mutability, ..}) = node.pat {
            let start = node.let_token.span();
            let end = mutability.map(|m| m.span())
                .unwrap_or_else(|| node.let_token.span());

            if mutability.is_some() && self.within_spans(start.start(), end.end()) {
                return self.set_help_between(start, end, HelpItem::LocalMut);
            }
        }

        token![self, node.let_token, Local];
    }];
    method![visit_macro(self, node: syn::Macro) {
        if self.within_spans(node.path.span().start(), node.bang_token.span().end()) {
            return self.set_help_between(node.path.span(), node.bang_token.span(), HelpItem::Macro);
        }
    }];
    // method![visit_macro_delimiter(self, node: syn::MacroDelimiter)];
    method![visit_member(self, node: syn::Member)];
    method![visit_meta(self, node: syn::Meta)];
    method![visit_meta_list(self, node: syn::MetaList)];
    method![visit_meta_name_value(self, node: syn::MetaNameValue)];
    method![visit_method_turbofish(self, node: syn::MethodTurbofish)];
    method![visit_nested_meta(self, node: syn::NestedMeta)];
    method![visit_parenthesized_generic_arguments(
        self,
        node: syn::ParenthesizedGenericArguments
    )];
    method![visit_pat(self, node: syn::Pat)];
    method![visit_pat_box(self, node: syn::PatBox)];
    method![visit_pat_ident(self, node: syn::PatIdent) {
        let result = match (&node.by_ref, &node.mutability) {
            (Some(by_ref), Some(..)) => Some((by_ref as &dyn Spanned, HelpItem::PatIdentRefMut)),
            (Some(by_ref), None) => Some((by_ref as &dyn Spanned, HelpItem::PatIdentRef)),
            (None, Some(mutability)) => Some((mutability as &dyn Spanned, HelpItem::PatIdentMut)),
            (None, None) => None
        };

        if let Some((start, item)) = result {
            if self.between(start, &node.ident) {
                self.set_help_between(start.span(), node.ident.span(), item);
            }
        }

    }];
    method![visit_pat_lit(self, node: syn::PatLit)];
    method![visit_pat_macro(self, node: syn::PatMacro)];
    method![visit_pat_or(self, node: syn::PatOr)];
    method![visit_pat_path(self, node: syn::PatPath)];
    method![visit_pat_range(self, node: syn::PatRange)];
    method![visit_pat_reference(self, node: syn::PatReference)];
    method![visit_pat_rest(self, node: syn::PatRest)];
    method![visit_pat_slice(self, node: syn::PatSlice)];
    method![visit_pat_struct(self, node: syn::PatStruct)];
    method![visit_pat_tuple(self, node: syn::PatTuple)];
    method![visit_pat_tuple_struct(self, node: syn::PatTupleStruct)];
    method![visit_pat_type(self, node: syn::PatType)];
    method![visit_pat_wild(self, node: syn::PatWild)];
    // TODO:
    // * turbofish foo::<sfd>
    // * Fn patterns: Fn(A, B) -> C
    method![visit_path(self, node: syn::Path) {

    }];
    method![visit_path_arguments(self, node: syn::PathArguments)];
    method![visit_path_segment(self, node: syn::PathSegment) {
        if node.ident == "self" {
            token![self, node.ident, PathSegmentSelf];
        } else if node.ident == "super" {
            token![self, node.ident, PathSegmentSuper];
        }
    }];
    method![visit_predicate_eq(self, node: syn::PredicateEq)];
    method![visit_predicate_lifetime(self, node: syn::PredicateLifetime)];
    method![visit_predicate_type(self, node: syn::PredicateType)];
    fn visit_qself(&mut self, node: &'ast syn::QSelf) {
        if let Some(as_token) = node.as_token {
            if self.within(&as_token) {
                return self
                    .set_help_span(self.ancestors.last().unwrap().span, HelpItem::QualifiedAs);
            }
        }
        syn::visit::visit_qself(self, node);
    }
    // fn visit_range_limits(&mut self, node: &'ast syn::RangeLimits) {
    method![@attrs visit_receiver(self, node: syn::Receiver) {
        let item = match (&node.reference, &node.mutability) {
            (Some(_), Some(_)) => HelpItem::MutSelf,
            (Some(_), None) => HelpItem::RefSelf,
            (None, Some(mutability)) => {
                token![self, mutability, FnArgMut];
                HelpItem::ValueSelf
            },
            (None, None) => HelpItem::ValueSelf
        };
        return self.set_help(&node, item);
    }];
    method![visit_return_type(self, node: syn::ReturnType) {
        if let syn::ReturnType::Type(rarrow, _) = node {
            token![self, rarrow, RArrow];
        }
    }];
    method![visit_signature(self, node: syn::Signature) {
        token![self, some node.asyncness, AsyncFn];
        token![self, some node.constness, ConstFn];
        token![self, some node.abi, FnAbi];
        token![self, some node.unsafety, UnsafeFn];
    }];
    method![visit_stmt(self, node: syn::Stmt)];
    method![visit_trait_bound(self, node: syn::TraitBound)];
    method![visit_trait_bound_modifier(
        self,
        node: syn::TraitBoundModifier
    )];
    method![visit_trait_item(self, node: syn::TraitItem)];
    method![visit_trait_item_const(self, node: syn::TraitItemConst) {
        token![self, node.const_token, TraitItemConst];
    }];
    method![visit_trait_item_macro(self, node: syn::TraitItemMacro)];
    method![visit_trait_item_method(self, node: syn::TraitItemMethod) {
        token![self, node.sig.fn_token, FnToken];
        token![self, node.sig.ident, TraitItemMethod];
    }];
    method![visit_trait_item_type(self, node: syn::TraitItemType) {
        token![self, node.type_token, TraitItemType];
    }];
    method![visit_type(self, node: syn::Type)];
    method![visit_type_array(self, node: syn::TypeArray)];
    method![visit_type_bare_fn(self, node: syn::TypeBareFn) @terminal {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        return self.set_help(node, HelpItem::FnTypeToken);
    }];
    method![visit_type_group(self, node: syn::TypeGroup)];
    method![visit_type_impl_trait(self, node: syn::TypeImplTrait) {
        token![self, node.impl_token, TypeImplTrait];
    }];
    method![visit_type_infer(self, node: syn::TypeInfer)];
    method![visit_type_macro(self, node: syn::TypeMacro)];
    method![visit_type_never(self, node: syn::TypeNever)];
    method![visit_type_param(self, node: syn::TypeParam)];
    method![visit_type_param_bound(self, node: syn::TypeParamBound)];
    method![visit_type_paren(self, node: syn::TypeParen)];
    method![visit_type_path(self, node: syn::TypePath)];
    method![visit_type_ptr(self, node: syn::TypePtr) {
        if let Some(const_token) = node.const_token {
            token![self, const_token, ConstPtr];
        }
        if let Some(mutability) = node.mutability {
            token![self, mutability, MutPtr];
        }
    }];
    method![visit_type_reference(self, node: syn::TypeReference) {
        let last_span = node.mutability.map(|t| t.span())
            .or_else(|| node.lifetime.as_ref().map(|t| t.span()))
            .unwrap_or_else(|| node.and_token.span());

        if self.within_spans(node.and_token.span().start(), last_span.end()) {
            return self.set_help(&node, HelpItem::TypeReference {
                lifetime: node.lifetime.is_some(),
                mutable: node.mutability.is_some()
            });
        }
    }];
    method![visit_type_slice(self, node: syn::TypeSlice)];
    method![visit_type_trait_object(self, node: syn::TypeTraitObject) {
        if let Some(dyn_token) = node.dyn_token {
            token![self, dyn_token, Dyn];
        }
        if let Some(plus_token) = node.bounds.pairs()
            .filter_map(|pair| pair.punct().cloned())
            .find(|punct| self.within(punct))
        {
            return self.set_help(&plus_token, HelpItem::TraitBound);
        }
    }];
    method![visit_type_tuple(self, node: syn::TypeTuple)];
    method![visit_un_op(self, node: syn::UnOp)];
    method![visit_use_glob(self, node: syn::UseGlob)];
    method![visit_use_group(self, node: syn::UseGroup)];
    method![visit_use_name(self, node: syn::UseName)];
    method![visit_use_path(self, node: syn::UsePath)];
    method![visit_use_rename(self, node: syn::UseRename) {
        token![self, node.as_token, AsRename];
    }];
    method![visit_use_tree(self, node: syn::UseTree)];
    method![visit_variadic(self, node: syn::Variadic)];
    method![visit_variant(self, node: syn::Variant)];
    method![visit_vis_crate(self, node: syn::VisCrate)];
    method![visit_vis_public(self, node: syn::VisPublic)];
    method![visit_vis_restricted(self, node: syn::VisRestricted)];
    method![visit_visibility(self, node: syn::Visibility) {
        match node {
            syn::Visibility::Public(vis) => token![self, vis.pub_token, VisPublic],
            syn::Visibility::Crate(vis) => token![self, vis.crate_token, VisCrate],
            syn::Visibility::Restricted(..) => token![self, node, VisRestricted],
            syn::Visibility::Inherited => {}
        }
    }];
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
