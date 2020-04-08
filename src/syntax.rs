use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use serde::Serialize;
use std::any::Any;
#[cfg(feature = "dev")]
use std::fmt::Debug;
use syn::spanned::Spanned;
use syn::visit::Visit;

std::thread_local! {
    static TEMPLATE: tinytemplate::TinyTemplate<'static> = init_template();
}

std::include!(concat!(env!("OUT_DIR"), "/help.rs"));

#[cfg(feature = "dev")]
use super::log;

#[cfg(feature = "dev")]
trait DynAncestor {
    fn as_any(&self) -> &dyn Any;
    fn as_debug(&self) -> &dyn Debug;
}

#[cfg(feature = "dev")]
impl<T: Debug + Any + 'static> DynAncestor for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
    fn as_debug(&self) -> &dyn Debug {
        self as &dyn Debug
    }
}

struct HelpData {
    template: &'static str,
    title: &'static str,
    book: Option<&'static str>,
    keyword: Option<&'static str>,
    std: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HelpItem {
    ItemUse,
    ItemUseCrate,
    ItemUseLeadingColon,
    Macro,
    MacroTokens,
    PatBox,
    PatIdent {
        mutability: bool,
        by_ref: bool,
        ident: String,
    },
    PatIdentSubPat {
        ident: String,
    },
    PatIdentMutableArg {
        ident: String,
    },
    PatOrLeading,
    PatOr,
    PatRange {
        closed: bool,
    },
    PatRest {
        of: &'static str,
    },
    PatStruct {
        empty: bool,
        bindings: Option<&'static str>,
    },
    PatTuple {
        bindings: Option<&'static str>,
    },
    PatTupleStruct {
        bindings: Option<&'static str>,
    },
    PatWild,
    Attribute {
        outer: bool,
    },
    ItemExternCrate,
    ItemFn,
    ItemInlineMod,
    ItemExternMod,
    Unknown,
    FnToken {
        of: &'static str,
        name: String,
    },
    TraitItemMethod,
    ImplItemMethod,
    AsRename,
    AsRenameExternCrate,
    AsCast,
    ExprClosureArguments,
    ExprClosureAsync,
    ExprClosureStatic,
    AsyncExpression,
    AsyncFn,
    AwaitExpression,
    Break {
        label: bool,
        expr: bool,
    },
    ImplItemConst,
    TraitItemConst,
    ItemConst,
    ConstParam,
    ConstFn,
    ExprContinue {
        label: bool,
    },
    VisPublic,
    VisCrate,
    VisRestricted,
    Variant {
        name: String,
        fields: Option<&'static str>,
    },
    VariantDiscriminant {
        name: String,
    },
    Else,
    ItemEnum {
        empty: bool,
    },
    ItemForeignModAbi,
    FnAbi,
    TypeBareFnAbi,
    True,
    False,
    LitByteStr,
    LitFloat {
        suffix: Option<String>,
        separators: bool,
    },
    LitInt {
        suffix: Option<String>,
        mode: Option<&'static str>,
        prefix: Option<String>,
        separators: bool,
    },
    LitStr,
    FnTypeToken,
    ExprForLoopToken,
    BoundLifetimes,
    BoundLifetimesTraitBound {
        lifetime: String,
        ty: String,
        multiple: bool,
    },
    IfLet,
    If,
    ItemImpl {
        trait_: bool,
    },
    ItemImplForTrait,
    ItemMacroRules {
        name: String,
    },
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
    MutSelf,
    ValueSelf,
    RefSelf,
    PathLeadingColon,
    QSelfAsToken,
    StaticMut,
    Static,
    TypeReference {
        mutable: bool,
        lifetime: bool,
        ty: String,
    },
    TypeSlice {
        dynamic: bool,
        ty: String,
    },
    TypeTraitObject {
        ty: String,
        lifetime: Option<String>,
        multiple: bool,
        dyn_: bool,
    },
    UseGlob,
    UseGroup {
        parent: String,
    },
    ExprReference {
        mutable: bool,
    },
    ExprReturn,
    PathSegmentSelf,
    ExprUnsafe,
    ForeignItemType,
    ImplItemType,
    ItemUnsafeImpl,
    ItemStruct {
        unit: bool,
        name: String,
    },
    ItemUnsafeTrait,
    ItemTrait,
    ItemType,
    ItemUnion,
    PathSegmentSuper,
    UnsafeFn,
    TraitBoundModifierQuestion {
        sized: bool,
    },
    TraitItemType,
    TypeArray,
    TypeInfer,
    TypeNever,
    TypeParamBoundAdd,
    TypeTupleUnit,
    TypeTuple,
    TypeConstPtr,
    TypeMutPtr,
    WhereClause,
    TypeBareUnsafeFn,
    ItemTraitAlias,
    Field {
        name: Option<String>,
        of: &'static str,
        of_name: String,
    },
    // TODO: we could rewrite the pattern and explain what's going on better
    FieldPatUnnamed {
        ident: String,
    },
    FieldPatShorthand {
        ident: String,
    },
    FieldValueShorthand {
        name: String,
    },
    FatArrow,
    DocBlock {
        outer: bool,
    },
    RArrow,
    ExprRangeHalfOpen {
        from: bool,
        to: bool,
    },
    ExprRangeClosed {
        from: bool,
        to: bool,
    },
    StaticLifetime,
}

impl HelpItem {
    pub fn message(&self) -> String {
        TEMPLATE.with(|tt| tt.render(self.data().template, &self).unwrap())
    }

    pub fn title(&self) -> &'static str {
        self.data().title
    }

    pub fn keyword(&self) -> Option<&'static str> {
        self.data().keyword
    }

    pub fn std(&self) -> Option<&'static str> {
        self.data().std
    }

    pub fn book(&self) -> Option<&'static str> {
        self.data().book
    }

    fn data(&self) -> HelpData {
        help_to_template_data(self)
    }
}

pub(super) struct IntersectionVisitor<'ast> {
    location: LineColumn,
    help: HelpItem,
    item_location: (LineColumn, LineColumn),
    ancestors: Vec<Ancestor<'ast>>,
    #[cfg(not(feature = "dev"))]
    ancestors2: Vec<&'ast dyn Any>,
    #[cfg(feature = "dev")]
    ancestors2: Vec<&'ast dyn DynAncestor>,
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
        Self {
            location,
            item_location: (location, location),
            help: HelpItem::Unknown,
            ancestors: vec![],
            ancestors2: vec![],
            next_ancestor: Ancestor::new(),
            #[cfg(feature = "dev")]
            debug: "".to_string(),
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
            #[cfg(feature = "dev")]
            debug: self.debug,
        }
    }

    pub fn visit_element(mut self, file: &'ast syn::File, idx: usize) -> VisitorResult {
        let mut ancestor = Ancestor::new();
        ancestor.attributes = &file.attrs[..];
        ancestor.span = file.span();
        self.ancestors.push(ancestor);
        self.ancestors2.push(file);

        if idx < file.attrs.len() {
            self.visit_attribute(&file.attrs[idx]);
        } else {
            self.visit_item(&file.items[idx - file.attrs.len()]);
        }
        VisitorResult {
            help: self.help,
            item_location: self.item_location,
            #[cfg(feature = "dev")]
            debug: self.debug,
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
        super::within_locations(loc, start, end)
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

    #[cfg(not(feature = "dev"))]
    fn get_ancestor<T: 'static>(&self, idx: usize) -> Option<&'ast T> {
        self.ancestors2
            .get(self.ancestors2.len() - idx)
            .and_then(|any| any.downcast_ref())
    }
    #[cfg(feature = "dev")]
    fn get_ancestor<T: 'static>(&self, idx: usize) -> Option<&'ast T> {
        self.ancestors2
            .get(self.ancestors2.len() - idx)
            .and_then(|any| any.as_any().downcast_ref())
    }
}

macro_rules! method {
    (@_spancheck, $self:ident, $node:ident, $ty:path) => {{
        if !$self.within(&$node) {
            return;
        }
        $self.next_ancestor.span = $node.span();
        #[cfg(feature = "dev")]
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
    (@_set_ancestor @_spancheck, $self:ident, $node:ident) => {
        let mut next_ancestor = Ancestor::new();
        std::mem::swap(&mut $self.next_ancestor, &mut next_ancestor);
        $self.ancestors.push(next_ancestor);
    };
    (@_set_ancestor @_nospancheck, $self:ident, $node:ident) => {
        $self.ancestors2.push($node);
    };
    (@_terminal, @_reachable) => {
        #[allow(unreachable_code)]
    };
    (@_terminal, $name:ident, $body:expr) => {
        $body;
        unreachable!(stringify!($name));
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body $body:expr => $after:expr, @ $spancheck:ident) => {
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@_debug, $name, $self, $node];
            method![@$spancheck, $self, $node, $ty];
            $body;
            method![@_set_ancestor @$spancheck, $self, $node];
            $self.ancestors2.push($node);
            syn::visit::$name($self, $node);
            $self.ancestors2.pop();
            $after;
        }
    };
    (@_impl, $name:ident, $self:ident, $node:ident, $ty:path, @_body @terminal $body:expr, @ $spancheck:ident) => {
        #[allow(unreachable_code)]
        fn $name(&mut $self, $node: &'ast $ty) {
            method![@_debug, $name, $self, $node];
            method![@$spancheck, $self, $node, $ty];
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
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, { () }] => (), @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, $body] => (), @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal method![@_attrs, $self, $node, $body], @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) $body:expr => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, $body] => $after, @_spancheck];
    };
    (@attrs $name:ident($self:ident, $node:ident: $ty:path) => $after:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body method![@_attrs, $self, $node, ()] => $after, @_spancheck];
    };
    ($name:ident($self:ident, $node:ident: $ty:path) @terminal $body:expr) => {
        method![@_impl, $name, $self, $node, $ty, @_body @terminal $body, @_spancheck];
    };
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

// TODO
// - operators: question mark, <<, >>, +, -, /, %, +=, -=, *=, /=, %=, ||, &&, range operators, [..]
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
                let this_idx = attributes
                    .iter()
                    .enumerate()
                    .find(|(_, attr)| std::ptr::eq(node, *attr))
                    .expect("attr in list")
                    .0;

                let last = attributes
                    .iter()
                    .enumerate()
                    .filter(|&(i, attr)| {
                        i >= this_idx && attr.path.is_ident("doc") && outer_attr(attr) == outer
                    })
                    .last()
                    .expect("last")
                    .1;

                let start = attributes
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|&(i, attr)| {
                        i <= this_idx && !(attr.path.is_ident("doc") && outer_attr(attr) == outer)
                    })
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
    // TODO: BoundLifetimes in function pointers
    // TODO: BoundLifetimes in predicates, `for<'a> Foo<'a>: Bar<'a>`
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
            if self.between_spans(node.if_token.span(), let_token.span()) {
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

        if self.between_spans(node.and_token.span(), last_span) {
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
    method![@attrs visit_field(self, node: syn::Field) => {
        if self.settled() {
            return;
        }
        return self.set_help(node, HelpItem::Field {
            name: node.ident.as_ref().map(|id| id.to_string()),
            of: "struct",
            of_name: "".to_string()
        });
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
    }];
    method![visit_fields(self, node: syn::Fields)];
    method![visit_fields_named(self, node: syn::FieldsNamed)];
    method![visit_fields_unnamed(self, node: syn::FieldsUnnamed)];
    method![visit_file(self, node: syn::File) {
        // TODO: shebang
        let mut ancestor = Ancestor::new();
        ancestor.attributes = &node.attrs[..];
        ancestor.span = node.span();
        self.ancestors.push(ancestor);
        self.ancestors2.push(node);

        for attr in &node.attrs {
            if self.within(attr) {
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
    }];
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

        if self.between_spans(node.static_token.span(), end) {
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
        if self.within(node.sig.fn_token) {
            if let Some(item_impl) = self.get_ancestor::<syn::ItemImpl>(2) {
                return self.set_help(&node.sig.fn_token, HelpItem::FnToken {
                    of: if item_impl.trait_.is_some() { "trait method" } else { "method" },
                    name: node.sig.ident.to_string()
                });
            }
        }
    }];
    method![visit_impl_item_type(self, node: syn::ImplItemType) {
        token![self, node.type_token, ImplItemType];
    }];
    method![visit_index(self, node: syn::Index)];
    method![visit_item(self, node: syn::Item)];
    method![visit_item_const(self, node: syn::ItemConst) {
        token![self, node.const_token, ItemConst];
    }];
    method![@attrs visit_item_enum(self, node: syn::ItemEnum) => {
        match self.help {
            HelpItem::Variant { ref mut name, .. } => {
                *name = node.ident.to_string();
                return;
            }
            _ => {}
        }
        token![self, node.enum_token => node.ident, * HelpItem::ItemEnum {
            empty: node.variants.is_empty()
        }];
    }];
    method![@attrs visit_item_extern_crate(self, node: syn::ItemExternCrate) {
        if let Some((as_token, _)) = node.rename {
            token![self, as_token, AsRenameExternCrate];
        }
        let start = vis_span(&node.vis).unwrap_or_else(|| node.extern_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemExternCrate);
    }];
    method![visit_item_fn(self, node: syn::ItemFn) {
        token![self, node.sig.ident, ItemFn];
        token![self, node.sig.fn_token, * HelpItem::FnToken { of: "function", name: node.sig.ident.to_string() }];
    }];
    method![visit_item_foreign_mod(self, node: syn::ItemForeignMod) {
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

        if self.between_spans(node.static_token.span(), end) {
            return self.set_help_between(node.static_token.span(), end, if node.mutability.is_some() {
                HelpItem::StaticMut
            } else {
                HelpItem::Static
            });
        }
    }];
    method![@attrs visit_item_struct(self, node: syn::ItemStruct) => {
        match self.help {
            HelpItem::Field { ref mut of, ref mut of_name, .. } => {
                *of = "struct";
                *of_name = node.ident.to_string();
            },
            _ if !self.settled() => {
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
            }
            _ => {}
        }
    }];
    method![visit_item_trait(self, node: syn::ItemTrait) {
        token![self, some node.unsafety, ItemUnsafeTrait];
        token![self, node.trait_token, ItemTrait];
    }];
    method![visit_item_trait_alias(self, node: syn::ItemTraitAlias) {
        token![self, node.trait_token, ItemTraitAlias];
    }];
    method![visit_item_type(self, node: syn::ItemType) {
        token![self, node.type_token => node.ident, ItemType];
    }];
    method![@attrs visit_item_union(self, node: syn::ItemUnion) => {
        match self.help {
            HelpItem::Field { ref mut of, ref mut of_name, .. } => {
                *of = "union";
                *of_name = node.ident.to_string();
                return;
            },
            _ if self.settled() => return,
            _ => {}
        }
        token![self, node.union_token, ItemUnion];
    }];
    method![@attrs visit_item_use(self, node: syn::ItemUse) {
        token![self, some node.leading_colon, ItemUseLeadingColon];

        let mut queue = vec![&node.tree];

        while let Some(use_tree) = queue.pop() {
            match use_tree {
                syn::UseTree::Path(use_path) => if use_path.ident == "crate" && self.between(&use_path.ident, &use_path.ident) {
                    return self.set_help(&use_path.ident, HelpItem::ItemUseCrate);
                },
                syn::UseTree::Group(use_group) => queue.extend(use_group.items.iter()),
                _ => {}
            }
        }
    } => {
        if self.settled() {
            return;
        }
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemUse);
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
    method![visit_lit_byte_str(self, node: syn::LitByteStr) @terminal {
        return self.set_help(node, HelpItem::LitByteStr);
    }];
    method![visit_lit_char(self, node: syn::LitChar)];
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
            prefix @ Some("0b") => (prefix, Some("binary")),
            prefix @ Some("0x") => (prefix, Some("hexadecimal")),
            prefix @ Some("0o") => (prefix, Some("octal")),
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
        return self.set_help(node, HelpItem::LitStr);
    }];
    method![visit_local(self, node: syn::Local) {
        if let syn::Pat::Ident(syn::PatIdent { mutability, ..}) = node.pat {
            let start = node.let_token.span();
            let end = mutability.map(|m| m.span())
                .unwrap_or_else(|| node.let_token.span());

            if mutability.is_some() && self.between_spans(start, end) {
                return self.set_help_between(start, end, HelpItem::LocalMut);
            }
        }

        token![self, node.let_token, Local];
    }];
    method![visit_macro(self, node: syn::Macro) {
        if self.between_spans(node.path.span(), node.bang_token.span()) {
            return self.set_help_between(node.path.span(), node.bang_token.span(), HelpItem::Macro);
        }
        token![self, node.tokens, MacroTokens];
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

        if node.mutability.is_some() && self.get_ancestor::<syn::PatType>(2).is_some() {
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
    method![visit_pat_lit(self, node: syn::PatLit)];
    method![visit_pat_macro(self, node: syn::PatMacro)];
    method![@attrs visit_pat_or(self, node: syn::PatOr) {
        token![self, some node.leading_vert, PatOrLeading];
        for pair in node.cases.pairs() {
            token![self, some pair.punct(), PatOr];
        }
    }];
    method![visit_pat_path(self, node: syn::PatPath)];
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
    method![visit_pat_reference(self, node: syn::PatReference)];
    // OMITTED: handled in the parent patterns
    // method![visit_pat_rest(self, node: syn::PatRest)];
    method![visit_pat_slice(self, node: syn::PatSlice)];
    method![@attrs visit_pat_struct(self, node: syn::PatStruct) {
        if node.fields.is_empty() {
            token![self, some node.dot2_token, * HelpItem::PatStruct {
                empty: true,
                bindings: pattern_bindings(&self)
            }];
        }
        token![self, some node.dot2_token, * HelpItem::PatRest {
            of: "struct"
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
    method![@attrs visit_pat_tuple(self, node: syn::PatTuple) {
        if self.get_ancestor::<syn::PatTupleStruct>(1).is_some() {
            return;
        }
    } => {
        if self.settled() {
            return;
        }

        if let Some(pat @ syn::Pat::Rest(..)) = node.elems.last() {
            token![self, pat, * HelpItem::PatRest { of: "tuple struct" }];
        }

        return self.set_help(node, HelpItem::PatTuple {
            bindings: pattern_bindings(&self)
        });
    }];
    method![@attrs visit_pat_tuple_struct(self, node: syn::PatTupleStruct) => {
        if self.settled() {
            return;
        }

        return self.set_help(node, HelpItem::PatTupleStruct {
            bindings: pattern_bindings(&self)
        });
    }];
    method![visit_pat_type(self, node: syn::PatType)];
    method![@attrs visit_pat_wild(self, node: syn::PatWild) @terminal {
        return self.set_help(node, HelpItem::PatWild);
    }];
    // TODO:
    // * turbofish foo::<sfd>
    // * Fn patterns: Fn(A, B) -> C
    method![visit_path(self, node: syn::Path) {
        token![self, some node.leading_colon, PathLeadingColon]
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
                    .set_help_span(self.ancestors.last().unwrap().span, HelpItem::QSelfAsToken);
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
                token![self, mutability, * HelpItem::PatIdentMutableArg {
                    ident: "self".to_string()
                }];
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
    method![visit_trait_item_const(self, node: syn::TraitItemConst) {
        token![self, node.const_token, TraitItemConst];
    }];
    method![visit_trait_item_macro(self, node: syn::TraitItemMacro)];
    method![visit_trait_item_method(self, node: syn::TraitItemMethod) {
        token![self, node.sig.fn_token, * HelpItem::FnToken { of: "trait method", name: node.sig.ident.to_string() }];
        token![self, node.sig.ident, TraitItemMethod];
    }];
    method![visit_trait_item_type(self, node: syn::TraitItemType) {
        token![self, node.type_token, TraitItemType];
    }];
    method![visit_type(self, node: syn::Type)];
    method![visit_type_array(self, node: syn::TypeArray) {
        if self.between_locations(node.span().start(), node.elem.span().start()) ||
            self.between_spans(node.semi_token.span(), node.span()) {
            return self.set_help(node, HelpItem::TypeArray);
        }
    }];
    method![visit_type_bare_fn(self, node: syn::TypeBareFn) @terminal {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        return self.set_help(node, HelpItem::FnTypeToken);
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
    method![visit_type_param(self, node: syn::TypeParam)];
    method![visit_type_param_bound(self, node: syn::TypeParamBound)];
    method![visit_type_paren(self, node: syn::TypeParen)];
    method![visit_type_path(self, node: syn::TypePath)];
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
    method![visit_type_reference(self, node: syn::TypeReference) => {
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
    method![visit_use_group(self, node: syn::UseGroup)];
    method![visit_use_name(self, node: syn::UseName)];
    method![visit_use_path(self, node: syn::UsePath) => {
        match &*node.tree {
            syn::UseTree::Group(use_group) => {
                if self.within(use_group) {
                    return self.set_help(use_group, HelpItem::UseGroup {
                        parent: node.ident.to_string()
                    });
                }
            },
            _ => {}
        }
    }];
    method![visit_use_rename(self, node: syn::UseRename) {
        token![self, node.as_token, AsRename];
    }];
    method![visit_use_tree(self, node: syn::UseTree)];
    method![visit_variadic(self, node: syn::Variadic)];
    method![@attrs visit_variant(self, node: syn::Variant) => {
        if !self.settled() {
            if let Some((eq_token, discriminant)) = &node.discriminant {
                if self.between(&eq_token, &discriminant) {
                    return self.set_help_between(eq_token.span(), discriminant.span(), HelpItem::VariantDiscriminant {
                        name: node.ident.to_string()
                    });
                }
            }
            return self.set_help(node, HelpItem::Variant {
                name: "".to_string(),
                fields: match node.fields {
                    syn::Fields::Named(..) => Some("named"),
                    syn::Fields::Unnamed(..) => Some("unnamed"),
                    syn::Fields::Unit => None
                }
            });
        }

        match self.help {
            HelpItem::Field { ref mut of, ref mut of_name, .. } => {
                *of = "enum variant";
                *of_name = node.ident.to_string();
            },
            _ => {}
        }
    }];
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

fn pattern_bindings(visitor: &IntersectionVisitor) -> Option<&'static str> {
    if visitor.get_ancestor::<syn::Local>(2).is_some() {
        return Some("let");
    }
    if visitor.get_ancestor::<syn::PatType>(2).is_some() {
        return Some("arg");
    }

    return None;
}
