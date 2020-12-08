use crate::help::*;
use crate::ir::{Location, NodeId, NodeMap, Owner, Range};
use crate::syn_wrappers::{Comment, Syn, SynKind};
use generics::Generics;
use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use syn::spanned::Spanned;

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
macro_rules! get_ancestor {
    ($self:ident, $ty:ident, $depth:expr) => {
        match $self.get_ancestor($depth) {
            Some(Syn::$ty(node)) => Some(node),
            _ => None,
        }
    };
}

mod expressions;
mod generics;
mod items;
mod nested_items;
mod patterns;
mod types;

#[derive(Default)]
pub struct ExplorationState {
    location_index: usize,
    comment_index: usize,
}

pub struct ExplorationIterator<'a, I> {
    pub analyzer: &'a Analyzer,
    pub state: &'a mut ExplorationState,
    pub source: I,
}

impl<'a, I: Iterator<Item = Location>> Iterator for ExplorationIterator<'a, I> {
    type Item = Option<AnalysisResult>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(location) = self.source.next() {
            self.analyzer
                .analyze_for_exploration(&mut self.state, location)
        } else {
            self.analyzer.analyze_comment(&mut self.state)
        }
    }
}

pub struct Analyzer {
    pub(crate) node_map: NodeMap,
    pub(crate) locations: Vec<(NodeId, Range)>,
    pub(crate) owner: Rc<Owner>,
}

struct NodeAnalyzer<'a> {
    id: NodeId,
    location: Location,
    analyzer: &'a Analyzer,
    ancestors: &'a [(NodeId, Syn<'a>)],
    generics_state: &'a mut GenericsState,
    help: Option<(Range, HelpItem)>,
}

#[derive(Default)]
struct GenericsState {
    generics: Vec<Generics>,
    from_item: HashMap<NodeId, usize>,
    from_node: HashMap<NodeId, usize>,
    stack: Vec<NodeId>,
}

pub struct AnalysisResult {
    pub start: Location,
    pub end: Location,
    pub help: HelpItem,
}

impl Analyzer {
    fn analyze_for_exploration(
        &self,
        state: &mut ExplorationState,
        location: Location,
    ) -> Option<Option<AnalysisResult>> {
        let &(id, range) = self.locations.get(state.location_index)?;

        if range.0 <= location && location <= range.1 {
            return Some(self.analyze_candidates_at(id, location, state.location_index, range));
        }

        for (idx, &(id, range)) in self.locations[state.location_index..].iter().enumerate() {
            if range.0 <= location && location <= range.1 {
                state.location_index += idx;
                return Some(self.analyze_candidates_at(id, location, state.location_index, range));
            }
        }

        None
    }

    fn analyze_comment(&self, state: &mut ExplorationState) -> Option<Option<AnalysisResult>> {
        let comments = &self.owner.1[..];
        if state.comment_index >= comments.len() {
            return None;
        }
        let start = state.comment_index;
        let comment = &self.owner.1.as_slice()[start..]
            .iter()
            .enumerate()
            .filter(|(_, comment)| comment.doc.is_none())
            .next();
        if let Some((idx, comment)) = comment {
            state.comment_index = idx + start + 1;
            Some(Some(AnalysisResult {
                start: comment.range.0,
                end: comment.range.1,
                help: HelpItem::Comment {
                    block: comment.block,
                },
            }))
        } else {
            None
        }
    }

    pub fn analyze(&self, location: Location) -> Option<AnalysisResult> {
        let loc_idx = self
            .locations
            .binary_search_by(|(_, range)| range.0.cmp(&location))
            .map(|idx| Some(idx))
            .unwrap_or_else(|err| if err == 0 { None } else { Some(err - 1) })?;

        let (id, range) = self.locations.get(loc_idx).cloned()?;

        self.analyze_candidates_at(id, location, loc_idx, range)
    }

    fn analyze_candidates_at(
        &self,
        id: NodeId,
        location: Location,
        loc_idx: usize,
        range: Range,
    ) -> Option<AnalysisResult> {
        let mut candidates = [None; 3];
        candidates[0] = Some(id);

        if location == range.0 && loc_idx > 0 {
            if let Some(&(prev_id, _)) = self
                .locations
                .get(loc_idx - 1)
                .filter(|(_, prev_range)| prev_range.1 == range.0)
            {
                candidates[1] = Some(prev_id);
            }
        }

        if location == range.1 {
            if let Some(&(next_id, _)) = self
                .locations
                .get(loc_idx + 1)
                .filter(|(_, next_range)| next_range.0 == range.1)
            {
                candidates[2] = Some(next_id);
            }
        }

        candidates.sort_by(|candidate, candidate2| {
            let (id1, id2) = match (candidate, candidate2) {
                (Some(id1), Some(id2)) => (*id1, *id2),
                _ => return std::cmp::Ordering::Equal,
            };

            self.descendant_of(id1, id2)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .iter()
            .filter_map(|id| *id)
            .flat_map(|id| self.analyze_node(id, location))
            .next()
    }

    fn analyze_node(&self, id: NodeId, location: Location) -> Option<AnalysisResult> {
        let ancestors = {
            let mut ancestors = vec![];
            let mut id = id;

            while let Some(node) = self.node_map.get(id) {
                ancestors.push((id, unsafe { node.element.as_syn() }));
                id = if let Some(parent_id) = node.parent() {
                    parent_id
                } else {
                    break;
                }
            }

            ancestors.reverse();

            ancestors
        };

        let mut generics_state = Default::default();

        for (idx, &(node_id, node)) in ancestors.iter().enumerate() {
            if node_id == id {
                continue;
            }

            let mut node_analyzer =
                NodeAnalyzer::new(node_id, location, &self, &mut generics_state);
            node_analyzer.ancestors = &ancestors[0..idx];

            node_analyzer.analyze_node_first_pass(node);

            if node_analyzer.help.is_some() {
                return node_analyzer.result();
            }
        }

        let mut ancestors = ancestors;
        while let Some((id, node)) = ancestors.pop() {
            let mut node_analyzer = NodeAnalyzer::new(id, location, &self, &mut generics_state);
            node_analyzer.ancestors = &ancestors[..];
            node_analyzer.analyze_node(node);

            if node_analyzer.help.is_some() {
                return node_analyzer.result();
            }
        }

        None
    }

    fn descendant_of(&self, child_id: NodeId, ancestor_id: NodeId) -> Option<std::cmp::Ordering> {
        let mut node_id = child_id;
        let mut ancestors = HashSet::new();

        loop {
            node_id = if let Some(parent_id) = self.node_map.get_parent(node_id) {
                parent_id
            } else {
                break;
            };

            ancestors.insert(node_id);

            if node_id == ancestor_id {
                return Some(std::cmp::Ordering::Less);
            }
        }

        node_id = ancestor_id;

        loop {
            node_id = if let Some(parent_id) = self.node_map.get_parent(node_id) {
                parent_id
            } else {
                break;
            };

            if node_id == child_id {
                return Some(std::cmp::Ordering::Greater);
            } else if ancestors.contains(&node_id) {
                return None;
            }
        }

        None
    }

    fn id_to_syn(&self, id: NodeId) -> Option<Syn> {
        self.node_map.id_to_syn(id)
    }

    fn syn_to_id(&self, syn: Syn) -> Option<NodeId> {
        self.node_map.syn_to_id(syn)
    }
}

impl<'a> NodeAnalyzer<'a> {
    fn new(
        id: NodeId,
        location: Location,
        analyzer: &'a Analyzer,
        generics_state: &'a mut GenericsState,
    ) -> NodeAnalyzer<'a> {
        NodeAnalyzer {
            id,
            location,
            analyzer,
            ancestors: &[],
            help: None,
            generics_state,
        }
    }

    fn result(self) -> Option<AnalysisResult> {
        self.help
            .map(|((start, end), help)| AnalysisResult { start, end, help })
    }

    fn analyze_node_first_pass(&mut self, node: Syn) {
        match node {
            Syn::ExprForLoop(i) => self.visit_expr_for_loop_first_pass(i),
            Syn::ItemEnum(i) => self.visit_item_enum_first_pass(i),
            Syn::ItemFn(i) => self.visit_item_fn_first_pass(i),
            Syn::ItemImpl(i) => self.visit_item_impl_first_pass(i),
            Syn::ItemStruct(i) => self.visit_item_struct_first_pass(i),
            Syn::ItemTrait(i) => self.visit_item_trait_first_pass(i),
            Syn::ItemTraitAlias(i) => self.visit_item_trait_alias_first_pass(i),
            Syn::ItemUnion(i) => self.visit_item_union_first_pass(i),
            Syn::ImplItemMethod(i) => self.visit_impl_item_method_first_pass(i),
            Syn::TraitItemMethod(i) => self.visit_trait_item_method_first_pass(i),
            Syn::Local(i) => self.visit_local_first_pass(i),
            Syn::VisRestricted(i) => self.visit_vis_restricted_first_pass(i),
            _ => {}
        }
    }

    fn analyze_node(&mut self, node: Syn) {
        match node {
            // OMITTED: handled upstream
            Syn::Abi(_i) => { /* self.visit_abi(i) */ }
            Syn::AngleBracketedGenericArguments(i) => {
                self.visit_angle_bracketed_generic_arguments(i)
            }
            Syn::Arm(i) => self.visit_arm(i),
            Syn::Attribute(i) => self.visit_attribute(i),
            Syn::AttrStyle(_i) => { /* self.visit_attr_style(i) */ }
            Syn::BareFnArg(_i) => { /* self.visit_bare_fn_arg(i) */ }
            Syn::BinOp(i) => self.visit_bin_op(i),
            Syn::Binding(i) => self.visit_binding(i),
            Syn::Block(_i) => { /* self.visit_block(i) */ }
            Syn::BoundLifetimes(_i) => { /* self.visit_bound_lifetimes(i) */ }
            Syn::ConstParam(i) => self.visit_const_param(i),
            Syn::Constraint(_i) => { /* self.visit_constraint(i) */ }
            Syn::Expr(_i) => { /* self.visit_expr(i) */ }
            Syn::ExprArray(i) => self.visit_expr_array(i),
            Syn::ExprAssign(i) => self.visit_expr_assign(i),
            Syn::ExprAssignOp(i) => self.visit_expr_assign_op(i),
            Syn::ExprAsync(i) => self.visit_expr_async(i),
            Syn::ExprAwait(i) => self.visit_expr_await(i),
            Syn::ExprBinary(_i) => { /* self.visit_expr_binary(i) */ }
            Syn::ExprBlock(_i) => { /* self.visit_expr_block(i) */ }
            Syn::ExprBox(i) => self.visit_expr_box(i),
            Syn::ExprBreak(i) => self.visit_expr_break(i),
            Syn::ExprCall(_i) => { /* self.visit_expr_call(i) */ }
            Syn::ExprCast(i) => self.visit_expr_cast(i),
            Syn::ExprClosure(i) => self.visit_expr_closure(i),
            Syn::ExprContinue(i) => self.visit_expr_continue(i),
            Syn::ExprField(i) => self.visit_expr_field(i),
            Syn::ExprForLoop(_i) => { /* self.visit_expr_for_loop(i) */ }
            Syn::ExprGroup(_i) => { /* self.visit_expr_group(i) */ }
            Syn::ExprIf(i) => self.visit_expr_if(i),
            Syn::ExprIndex(i) => self.visit_expr_index(i),
            Syn::ExprLet(_i) => { /* self.visit_expr_let(i) */ }
            Syn::ExprLit(_i) => { /* self.visit_expr_lit(i) */ }
            Syn::ExprLoop(i) => self.visit_expr_loop(i),
            Syn::ExprMacro(_i) => { /* self.visit_expr_macro(i) */ }
            Syn::ExprMatch(i) => self.visit_expr_match(i),
            Syn::ExprMethodCall(_i) => { /* self.visit_expr_method_call(i) */ }
            Syn::ExprParen(_i) => { /* self.visit_expr_paren(i) */ }
            Syn::ExprPath(_i) => { /* self.visit_expr_path(i) */ }
            Syn::ExprRange(i) => self.visit_expr_range(i),
            Syn::ExprReference(i) => self.visit_expr_reference(i),
            Syn::ExprRepeat(i) => self.visit_expr_repeat(i),
            Syn::ExprReturn(i) => self.visit_expr_return(i),
            Syn::ExprStruct(i) => self.visit_expr_struct(i),
            Syn::ExprTry(i) => self.visit_expr_try(i),
            Syn::ExprTryBlock(i) => self.visit_expr_try_block(i),
            Syn::ExprTuple(i) => self.visit_expr_tuple(i),
            Syn::ExprType(i) => self.visit_expr_type(i),
            Syn::ExprUnary(_i) => { /* self.visit_expr_unary(i) */ }
            Syn::ExprUnsafe(i) => self.visit_expr_unsafe(i),
            Syn::ExprWhile(i) => self.visit_expr_while(i),
            Syn::ExprYield(i) => self.visit_expr_yield(i),
            Syn::Field(i) => self.visit_field(i),
            Syn::FieldPat(i) => self.visit_field_pat(i),
            Syn::FieldValue(i) => self.visit_field_value(i),
            Syn::Fields(_i) => { /* self.visit_fields(i) */ }
            Syn::FieldsNamed(_i) => { /* self.visit_fields_named(i) */ }
            Syn::FieldsUnnamed(_i) => { /* self.visit_fields_unnamed(i) */ }
            Syn::File(i) => self.visit_file(i),
            Syn::FnArg(i) => self.visit_fn_arg(i),
            Syn::ForeignItem(_i) => { /* self.visit_foreign_item(i) */ }
            Syn::ForeignItemFn(_i) => { /* self.visit_foreign_item_fn(i) */ }
            Syn::ForeignItemMacro(_i) => { /* self.visit_foreign_item_macro(i) */ }
            Syn::ForeignItemStatic(i) => self.visit_foreign_item_static(i),
            Syn::ForeignItemType(i) => self.visit_foreign_item_type(i),
            Syn::GenericArgument(_i) => { /* self.visit_generic_argument(i) */ }
            Syn::GenericMethodArgument(_i) => { /* self.visit_generic_method_argument(i) */ }
            Syn::GenericParam(i) => self.visit_generic_param(i),
            Syn::Generics(_i) => { /* self.visit_generics(i) */ }
            Syn::Ident(i) => self.visit_ident(i),
            Syn::ImplItem(_i) => { /* self.visit_impl_item(i) */ }
            Syn::ImplItemConst(i) => self.visit_impl_item_const(i),
            Syn::ImplItemMacro(_i) => { /* self.visit_impl_item_macro(i) */ }
            Syn::ImplItemMethod(i) => self.visit_impl_item_method(i),
            Syn::ImplItemType(i) => self.visit_impl_item_type(i),
            Syn::Index(_i) => { /* self.visit_index(i) */ }
            Syn::Item(_i) => { /* self.visit_item(i) */ }
            Syn::ItemConst(i) => self.visit_item_const(i),
            Syn::ItemEnum(i) => self.visit_item_enum(i),
            Syn::ItemExternCrate(i) => self.visit_item_extern_crate(i),
            Syn::ItemFn(i) => self.visit_item_fn(i),
            Syn::ItemForeignMod(i) => self.visit_item_foreign_mod(i),
            Syn::ItemImpl(i) => self.visit_item_impl(i),
            Syn::ItemMacro(i) => self.visit_item_macro(i),
            Syn::ItemMacro2(_i) => { /* self.visit_item_macro2(i) */ }
            Syn::ItemMod(i) => self.visit_item_mod(i),
            Syn::ItemStatic(i) => self.visit_item_static(i),
            Syn::ItemStruct(i) => self.visit_item_struct(i),
            Syn::ItemTrait(i) => self.visit_item_trait(i),
            Syn::ItemTraitAlias(i) => self.visit_item_trait_alias(i),
            Syn::ItemType(i) => self.visit_item_type(i),
            Syn::ItemUnion(i) => self.visit_item_union(i),
            Syn::ItemUse(i) => self.visit_item_use(i),
            Syn::Label(i) => self.visit_label(i),
            Syn::Lifetime(i) => self.visit_lifetime(i),
            Syn::LifetimeDef(_i) => { /* self.visit_lifetime_def(i) */ }
            Syn::Lit(_i) => { /* self.visit_lit(i) */ }
            Syn::LitBool(i) => self.visit_lit_bool(i),
            Syn::LitByte(i) => self.visit_lit_byte(i),
            Syn::LitByteStr(i) => self.visit_lit_byte_str(i),
            Syn::LitChar(i) => self.visit_lit_char(i),
            Syn::LitFloat(i) => self.visit_lit_float(i),
            Syn::LitInt(i) => self.visit_lit_int(i),
            Syn::LitStr(i) => self.visit_lit_str(i),
            Syn::Local(_i) => { /* self.visit_local(i) */ }
            Syn::Macro(i) => self.visit_macro(i),
            Syn::Member(_i) => { /* self.visit_member(i) */ }
            Syn::MethodTurbofish(i) => self.visit_method_turbofish(i),
            Syn::ParenthesizedGenericArguments(_i) => { /* self.visit_parenthesized_generic_arguments(i) */
            }
            Syn::Pat(_i) => { /* self.visit_pat(i) */ }
            Syn::PatBox(i) => self.visit_pat_box(i),
            Syn::PatIdent(i) => self.visit_pat_ident(i),
            Syn::PatLit(_i) => { /* self.visit_pat_lit(i) */ }
            Syn::PatMacro(_i) => { /* self.visit_pat_macro(i) */ }
            Syn::PatOr(i) => self.visit_pat_or(i),
            Syn::PatPath(_i) => { /* self.visit_pat_path(i) */ }
            Syn::PatRange(i) => self.visit_pat_range(i),
            Syn::PatReference(_i) => { /* self.visit_pat_reference(i) */ }
            Syn::PatRest(_i) => { /* self.visit_pat_rest(i) */ }
            Syn::PatSlice(_i) => { /* self.visit_pat_slice(i) */ }
            Syn::PatStruct(i) => self.visit_pat_struct(i),
            Syn::PatTuple(i) => self.visit_pat_tuple(i),
            Syn::PatTupleStruct(i) => self.visit_pat_tuple_struct(i),
            Syn::PatType(_i) => { /* self.visit_pat_type(i) */ }
            Syn::PatWild(i) => self.visit_pat_wild(i),
            Syn::Path(i) => self.visit_path(i),
            Syn::PathArguments(_i) => { /* self.visit_path_arguments(i) */ }
            Syn::PathSegment(i) => self.visit_path_segment(i),
            Syn::PredicateEq(_i) => { /* self.visit_predicate_eq(i) */ }
            Syn::PredicateLifetime(_i) => { /* self.visit_predicate_lifetime(i) */ }
            Syn::PredicateType(i) => self.visit_predicate_type(i),
            Syn::QSelf(i) => self.visit_qself(i),
            Syn::Receiver(_i) => { /* self.visit_receiver(i) */ }
            Syn::ReturnType(i) => self.visit_return_type(i),
            Syn::Signature(i) => self.visit_signature(i),
            Syn::Stmt(_i) => { /* self.visit_stmt(i) */ }
            Syn::TraitBound(i) => self.visit_trait_bound(i),
            Syn::TraitBoundModifier(_i) => { /* self.visit_trait_bound_modifier(i) */ }
            Syn::TraitItem(_i) => { /* self.visit_trait_item(i) */ }
            Syn::TraitItemConst(i) => self.visit_trait_item_const(i),
            Syn::TraitItemMacro(_i) => { /* self.visit_trait_item_macro(i) */ }
            Syn::TraitItemMethod(i) => self.visit_trait_item_method(i),
            Syn::TraitItemType(i) => self.visit_trait_item_type(i),
            Syn::Type(_i) => { /* self.visit_type(i) */ }
            Syn::TypeArray(i) => self.visit_type_array(i),
            Syn::TypeBareFn(i) => self.visit_type_bare_fn(i),
            Syn::TypeGroup(_i) => { /* self.visit_type_group(i) */ }
            Syn::TypeImplTrait(i) => self.visit_type_impl_trait(i),
            Syn::TypeInfer(i) => self.visit_type_infer(i),
            Syn::TypeMacro(_i) => { /* self.visit_type_macro(i) */ }
            Syn::TypeNever(i) => self.visit_type_never(i),
            Syn::TypeParam(i) => self.visit_type_param(i),
            Syn::TypeParamBound(_i) => { /* self.visit_type_param_bound(i) */ }
            Syn::TypeParen(_i) => { /* self.visit_type_paren(i) */ }
            Syn::TypePath(i) => self.visit_type_path(i),
            Syn::TypePtr(i) => self.visit_type_ptr(i),
            Syn::TypeReference(i) => self.visit_type_reference(i),
            Syn::TypeSlice(i) => self.visit_type_slice(i),
            Syn::TypeTraitObject(i) => self.visit_type_trait_object(i),
            Syn::TypeTuple(i) => self.visit_type_tuple(i),
            Syn::UnOp(_i) => { /* self.visit_un_op(i) */ }
            Syn::UseGlob(i) => self.visit_use_glob(i),
            Syn::UseGroup(i) => self.visit_use_group(i),
            Syn::UseName(i) => self.visit_use_name(i),
            Syn::UsePath(i) => self.visit_use_path(i),
            Syn::UseRename(i) => self.visit_use_rename(i),
            Syn::UseTree(_i) => { /* self.visit_use_tree(i) */ }
            Syn::Variadic(_i) => { /* self.visit_variadic(i) */ }
            Syn::Variant(i) => self.visit_variant(i),
            Syn::VisCrate(i) => self.visit_vis_crate(i),
            Syn::VisPublic(i) => self.visit_vis_public(i),
            Syn::VisRestricted(_i) => { /* self.visit_vis_restricted(i) */ }
            Syn::Visibility(_i) => { /* self.visit_visibility(i) */ }
            Syn::WhereClause(i) => self.visit_where_clause(i),
            Syn::WherePredicate(_i) => { /* self.visit_where_predicate(i) */ }
            Syn::Comment(i) => self.visit_comment(i),
        }
    }

    fn set_help<S: Spanned>(&mut self, node: S, item: HelpItem) {
        self.set_help_between(node.span(), node.span(), item);
    }

    fn set_help_between(
        &mut self,
        start: proc_macro2::Span,
        end: proc_macro2::Span,
        item: HelpItem,
    ) {
        self.help = Some(((start.start().into(), end.end().into()), item));
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
        within_locations(
            LineColumn {
                line: loc.line,
                column: loc.column,
            },
            start,
            end,
        )
    }

    fn has_ancestor(&self, ancestor: usize, kind: SynKind) -> bool {
        self.get_ancestor(ancestor)
            .map(|node| node.kind() == kind)
            .unwrap_or(false)
    }

    fn get_ancestor(&self, ancestor: usize) -> Option<Syn> {
        self.ancestors
            .get(self.ancestors.len() - ancestor)
            .map(|(_, node)| *node)
    }

    fn get_ancestor_id(&self, ancestor: usize) -> Option<NodeId> {
        self.ancestors
            .get(self.ancestors.len() - ancestor)
            .map(|(node_id, _)| *node_id)
    }

    fn id_to_syn(&self, id: NodeId) -> Option<Syn> {
        self.analyzer.id_to_syn(id)
    }

    fn syn_to_id(&self, syn: Syn) -> Option<NodeId> {
        self.analyzer.syn_to_id(syn)
    }

    fn fill_generics_info(&mut self, item_id: NodeId, generics: &syn::Generics, reset: bool) {
        if self.generics_state.from_item.contains_key(&item_id) {
            return;
        }

        let node_id = if let Some(node_id) = self.syn_to_id(generics.into()) {
            node_id
        } else {
            return;
        };

        if let Some(generics) = self.analyze_generics(generics) {
            if reset {
                self.generics_state.stack = vec![item_id];
            } else {
                self.generics_state.stack.push(item_id);
            }

            self.generics_state.generics.push(generics);
            let id = self.generics_state.generics.len() - 1;
            self.generics_state.from_item.insert(item_id, id);
            self.generics_state.from_node.insert(node_id, id);
        } else {
            if reset {
                self.generics_state.stack = vec![];
            }
        }
    }

    fn generics_for(&self, id: NodeId) -> Option<&Generics> {
        self.generics_state
            .from_item
            .get(&id)
            .and_then(|&idx| self.generics_state.generics.get(idx))
    }

    //============= VISIT METHODS

    fn visit_angle_bracketed_generic_arguments(
        &mut self,
        node: &syn::AngleBracketedGenericArguments,
    ) {
        if node.colon2_token.is_some() {
            return self.set_help(node, HelpItem::Turbofish);
        }
    }
    fn visit_arm(&mut self, node: &syn::Arm) {
        token![self, node.fat_arrow_token, FatArrow];
        if let Some((if_token, _)) = node.guard {
            token![self, if_token, ArmIfGuard];
        }
    }
    fn visit_attribute(&mut self, node: &syn::Attribute) {
        let is_comment = |id| {
            self.analyzer
                .node_map
                .get(id)
                .into_iter()
                .flat_map(|n| n.children.iter())
                .filter_map(|&child_id| self.analyzer.node_map.get(child_id))
                .any(|child_node| child_node.element.kind() == SynKind::Comment)
        };

        if !node.path.is_ident("doc") || !is_comment(self.id) {
            return self.visit_explicit_attribute(node);
        }

        let outer = outer_attr(node);

        let parent_id = self.analyzer.node_map.get_parent(self.id);

        let parent_node = parent_id.and_then(|parent_id| self.analyzer.node_map.get(parent_id));

        let parent = parent_id.and_then(|parent_id| self.id_to_syn(parent_id));

        let attributes = parent
            .as_ref()
            .and_then(|parent| parent.attributes())
            .unwrap_or(&[]);

        if attributes.len() == 0 {
            return;
        }

        let attribute_ids: Vec<NodeId> = {
            let mut ids = vec![];

            let node_to_id = parent_node
                .into_iter()
                .flat_map(|n| &n.children)
                .filter_map(|&id| self.id_to_syn(id).map(|syn| (syn.data(), id)))
                .collect::<HashMap<_, _>>();

            for attr in attributes {
                if let Some(id) = node_to_id.get(&Syn::Attribute(attr).data()) {
                    ids.push(*id);
                } else {
                    #[cfg(feature = "dev")]
                    {
                        panic!("Unable to find id");
                    }
                    return;
                };
            }

            ids
        };

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
                i >= this_idx
                    && attr.path.is_ident("doc")
                    && outer_attr(attr) == outer
                    && is_comment(attribute_ids[i])
            })
            .last()
            .expect("last")
            .1;

        let start = attributes
            .iter()
            .enumerate()
            .rev()
            .filter(|&(i, attr)| {
                i <= this_idx
                    && !(attr.path.is_ident("doc")
                        && outer_attr(attr) == outer
                        && is_comment(attribute_ids[i]))
            })
            .next()
            .map(|(i, _)| &attributes[i - 1])
            .unwrap_or(&attributes[0]);

        let start_span = start.span();
        let end_span = last.span();

        return self.set_help_between(start_span, end_span, HelpItem::DocBlock { outer });
    }

    fn visit_bin_op(&mut self, node: &syn::BinOp) {
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
    }
    fn visit_binding(&mut self, node: &syn::Binding) {
        return self.set_help(
            node,
            HelpItem::Binding {
                ident: node.ident.to_string(),
            },
        );
    }
    fn visit_field(&mut self, node: &syn::Field) {
        let field_data = loop {
            if let Some(variant) = get_ancestor![self, Variant, 3] {
                break Some((FieldOf::Variant, variant.ident.to_string()));
            }

            if let Some(item_struct) = get_ancestor![self, ItemStruct, 3] {
                break Some((FieldOf::Struct, item_struct.ident.to_string()));
            }

            if let Some(item_union) = get_ancestor![self, ItemUnion, 2] {
                break Some((FieldOf::Union, item_union.ident.to_string()));
            }

            break None;
        };

        if let Some((field_of, of_name)) = field_data {
            return self.set_help(
                node,
                HelpItem::Field {
                    name: node.ident.as_ref().map(|id| id.to_string()),
                    of: field_of,
                    of_name,
                },
            );
        }
    }
    fn visit_field_pat(&mut self, node: &syn::FieldPat) {
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

        if let (None, syn::Pat::Ident(pat)) = (node.colon_token, &*node.pat) {
            // We need to replicate visit_pat_ident partially, because the pattern node is not
            // inserted (see visit_pat_struct in IR generation)

            self.visit_simple_pat_ident(pat);

            if self.help.is_some() {
                return;
            }

            return self.set_help(
                node,
                HelpItem::FieldPatShorthand {
                    ident: pat.ident.to_string(),
                    mutability: pat.mutability.is_some(),
                    by_ref: pat.by_ref.is_some(),
                },
            );
        }
    }
    fn visit_field_value(&mut self, node: &syn::FieldValue) {
        match (node.colon_token, &node.member) {
            (None, syn::Member::Named(ident)) => {
                return self.set_help(
                    node,
                    HelpItem::FieldValueShorthand {
                        name: ident.to_string(),
                    },
                );
            }
            _ => {}
        }

        if let syn::Member::Unnamed(..) = node.member {
            return self.set_help(node, HelpItem::FieldUnnamedValue);
        }
    }
    fn visit_file(&mut self, node: &syn::File) {
        // TODO: handle shebang in exploration
        // TODO: assign proper span to shebang
        token![self, some node.shebang, Shebang];
    }
    fn visit_fn_arg(&mut self, node: &syn::FnArg) {
        if let Some(sig) = get_ancestor![self, Signature, 1] {
            if sig
                .inputs
                .first()
                .map(|arg| std::ptr::eq(arg, node))
                .unwrap_or(false)
            {
                if let Some(item) = receiver_help(sig) {
                    return self.set_help(node, item);
                }
            }
        }
    }
    fn visit_foreign_item_static(&mut self, node: &syn::ForeignItemStatic) {
        let end = node
            .mutability
            .as_ref()
            .map(Spanned::span)
            .unwrap_or_else(|| node.static_token.span());

        if self.between_spans(node.static_token.span(), end) {
            return self.set_help_between(
                node.static_token.span(),
                end,
                if node.mutability.is_some() {
                    HelpItem::StaticMut
                } else {
                    HelpItem::Static
                },
            );
        }
    }
    fn visit_foreign_item_type(&mut self, node: &syn::ForeignItemType) {
        token![self, node.type_token, ForeignItemType];
    }
    fn visit_ident(&mut self, node: &proc_macro2::Ident) {
        let raw = node.to_string();

        let start = node.span().start();
        if raw.starts_with("r#")
            && self.between_locations(
                start,
                LineColumn {
                    column: start.column + 2,
                    ..start
                },
            )
        {
            return self.set_help(node, HelpItem::RawIdent);
        }
    }
    fn visit_label(&mut self, node: &syn::Label) {
        let loop_of = if get_ancestor![self, ExprLoop, 1].is_some() {
            LoopOf::Loop
        } else if get_ancestor![self, ExprForLoop, 1].is_some() {
            LoopOf::For
        } else if get_ancestor![self, ExprBlock, 1].is_some() {
            LoopOf::Block
        } else {
            // Handled in ExprWhile
            return;
        };

        return self.set_help(&node, HelpItem::Label { loop_of });
    }
    fn visit_lifetime(&mut self, node: &syn::Lifetime) {
        if node.ident == "static" {
            return self.set_help(node, HelpItem::StaticLifetime);
        }
    }
    fn visit_lit_bool(&mut self, node: &syn::LitBool) {
        return self.set_help(
            node,
            if node.value {
                HelpItem::True
            } else {
                HelpItem::False
            },
        );
    }
    fn visit_lit_byte(&mut self, node: &syn::LitByte) {
        return self.set_help(node, HelpItem::LitByte);
    }
    fn visit_lit_byte_str(&mut self, node: &syn::LitByteStr) {
        let prefix = raw_string_literal(node.to_token_stream().to_string(), "br");
        let raw = prefix.is_some();
        return self.set_help(node, HelpItem::LitByteStr { raw, prefix });
    }
    fn visit_lit_char(&mut self, node: &syn::LitChar) {
        return self.set_help(node, HelpItem::LitChar);
    }
    fn visit_lit_float(&mut self, node: &syn::LitFloat) {
        let raw = node.to_string();
        let suffix = Some(node.suffix())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let separators = raw.chars().any(|c| c == '_');

        return self.set_help(node, HelpItem::LitFloat { suffix, separators });
    }
    fn visit_lit_int(&mut self, node: &syn::LitInt) {
        if self.has_ancestor(4, SynKind::TypeArray) {
            return;
        }

        let raw = node.to_string();

        let suffix = Some(node.suffix())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let separators = raw.chars().any(|c| c == '_');

        let (prefix, mode) = match raw.get(0..2) {
            prefix @ Some("0b") => (prefix, Some(IntMode::Binary)),
            prefix @ Some("0x") => (prefix, Some(IntMode::Hexadecimal)),
            prefix @ Some("0o") => (prefix, Some(IntMode::Octal)),
            _ => (None, None),
        };

        return self.set_help(
            node,
            HelpItem::LitInt {
                mode,
                separators,
                suffix,
                prefix: prefix.map(|s| s.to_string()),
            },
        );
    }
    fn visit_lit_str(&mut self, node: &syn::LitStr) {
        let prefix = raw_string_literal(node.to_token_stream().to_string(), "r");
        let raw = prefix.is_some();
        return self.set_help(node, HelpItem::LitStr { raw, prefix });
    }
    fn visit_local_first_pass(&mut self, node: &syn::Local) {
        let ident_pat = match &node.pat {
            syn::Pat::Ident(pat) => Some(pat),
            syn::Pat::Type(syn::PatType { pat, .. }) => match &**pat {
                syn::Pat::Ident(pat_ident) => Some(pat_ident),
                _ => None,
            },
            _ => None,
        };

        match ident_pat {
            Some(syn::PatIdent {
                ident, mutability, ..
            }) => {
                token![self, node.let_token => ident, * HelpItem::Local {
                    mutability: mutability.is_some(),
                    ident: Some(ident.to_string())
                }];
            }
            _ => {
                token![
                    self,
                    node.let_token,
                    *HelpItem::Local {
                        mutability: false,
                        ident: None
                    }
                ];
            }
        }
    }
    fn visit_macro(&mut self, node: &syn::Macro) {
        if get_ancestor![self, ItemMacro, 1]
            .map(|item| item.ident.is_some())
            .unwrap_or(false)
        {
            return;
        }

        if self.between_spans(node.path.span(), node.bang_token.span()) {
            return self.set_help_between(
                node.path.span(),
                node.bang_token.span(),
                HelpItem::Macro,
            );
        }
        token![self, node.tokens, MacroTokens];
    }
    fn visit_method_turbofish(&mut self, node: &syn::MethodTurbofish) {
        return self.set_help(node, HelpItem::Turbofish);
    }
    fn visit_path(&mut self, node: &syn::Path) {
        let qself = loop {
            let ancestor = if let Some(ancestor) = self
                .analyzer
                .node_map
                .get_parent(self.id)
                .and_then(|parent_id| self.id_to_syn(parent_id))
            {
                ancestor
            } else {
                return;
            };

            break match ancestor {
                Syn::ExprPath(expr_path) => &expr_path.qself,
                Syn::TypePath(type_path) => &type_path.qself,
                Syn::PatPath(pat_path) => &pat_path.qself,
                _ => &None,
            };
        };

        let simple_qself = qself
            .as_ref()
            .map(|q| q.as_token.is_some())
            .unwrap_or(false);

        if special_path_help(
            self,
            node.leading_colon.filter(|_| simple_qself),
            node.segments.first().map(|s| &s.ident),
            node.segments.len() == 1 && self.has_ancestor(1, SynKind::ExprPath),
        ) {
            return;
        }
    }
    fn visit_path_segment(&mut self, node: &syn::PathSegment) {
        if node.ident == "super" {
            return self.set_help(&node.ident, HelpItem::PathSegmentSuper);
        }
        if let syn::PathArguments::Parenthesized(..) = node.arguments {
            return self.set_help(node, HelpItem::ParenthesizedGenericArguments);
        }
    }
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
    fn visit_qself(&mut self, node: &syn::QSelf) {
        return self.set_help_between(
            node.lt_token.span(),
            node.gt_token.span(),
            HelpItem::QSelf {
                as_trait: node.as_token.is_some(),
            },
        );
    }
    fn visit_return_type(&mut self, node: &syn::ReturnType) {
        let rarrow = if let syn::ReturnType::Type(rarrow, _) = node {
            Some(rarrow)
        } else {
            None
        };

        token![self, some rarrow, * HelpItem::RArrow {
            return_of: if get_ancestor![self, TypeBareFn, 1].is_some() {
                ReturnOf::BareFunctionType
            } else if get_ancestor![self, ExprClosure, 1].is_some() {
                ReturnOf::Closure
            } else if get_ancestor![self, ParenthesizedGenericArguments, 1].is_some() {
                ReturnOf::FnTrait
            } else {
                // TODO: distinguish methods
                ReturnOf::Function
            }
        }];
    }
    fn visit_signature(&mut self, node: &syn::Signature) {
        token![self, some node.asyncness, AsyncFn];
        token![self, some node.constness, ConstFn];
        token![self, some node.abi, FnAbi];
        token![self, some node.unsafety, UnsafeFn];
    }
    fn visit_trait_bound(&mut self, node: &syn::TraitBound) {
        if let Some((bound_lifetimes, lifetime, multiple)) = node
            .lifetimes
            .as_ref()
            .filter(|bound_lifetimes| self.within(bound_lifetimes))
            .and_then(|bound_lifetimes| {
                bound_lifetimes
                    .lifetimes
                    .first()
                    .map(|lt| (bound_lifetimes, lt, bound_lifetimes.lifetimes.len() > 1))
            })
        {
            return self.set_help(
                bound_lifetimes,
                HelpItem::BoundLifetimesTraitBound {
                    lifetime: lifetime.lifetime.to_string(),
                    multiple,
                    ty: node.path.to_token_stream().to_string(),
                },
            );
        }
        if let syn::TraitBoundModifier::Maybe(..) = node.modifier {
            return self.set_help(
                node,
                HelpItem::TraitBoundModifierQuestion {
                    sized: node.path.is_ident("Sized"),
                },
            );
        }
    }
    fn visit_use_glob(&mut self, node: &syn::UseGlob) {
        return self.set_help(node, HelpItem::UseGlob);
    }
    fn visit_use_group(&mut self, node: &syn::UseGroup) {
        if let Some(path) = get_ancestor![self, UsePath, 2] {
            let parent = path.ident.to_string();
            return self.set_help(node, HelpItem::UseGroup { parent });
        }
    }
    fn visit_use_name(&mut self, node: &syn::UseName) {
        if node.ident != "self" {
            return;
        }

        if get_ancestor![self, UseGroup, 2].is_none() {
            return;
        }

        let parent = if let Some(path) = get_ancestor![self, UsePath, 4] {
            path.ident.to_string()
        } else {
            return;
        };

        return self.set_help(node, HelpItem::UseGroupSelf { parent });
    }
    fn visit_use_path(&mut self, node: &syn::UsePath) {
        let mut root_path = true;

        for i in (2..).step_by(2) {
            if get_ancestor![self, UseGroup, i].is_some() {
                continue;
            }

            root_path = get_ancestor![self, ItemUse, i].is_some();
            break;
        }

        if root_path && special_path_help(self, None, Some(&node.ident), false) {
            return;
        }

        if node.ident == "super" {
            return self.set_help(&node.ident, HelpItem::PathSegmentSuper);
        }
    }
    fn visit_use_rename(&mut self, node: &syn::UseRename) {
        token![self, node.as_token, AsRename];
    }
    fn visit_variant(&mut self, node: &syn::Variant) {
        if let Some((eq_token, discriminant)) = &node.discriminant {
            if self.between(&eq_token, &discriminant) {
                return self.set_help_between(
                    eq_token.span(),
                    discriminant.span(),
                    HelpItem::VariantDiscriminant {
                        name: node.ident.to_string(),
                    },
                );
            }
        }
        let name = if let Some(item_enum) = get_ancestor![self, ItemEnum, 1] {
            item_enum.ident.to_string()
        } else {
            return;
        };

        return self.set_help(
            node,
            HelpItem::Variant {
                name,
                fields: match node.fields {
                    syn::Fields::Named(..) => Some(Fields::Named),
                    syn::Fields::Unnamed(..) => Some(Fields::Unnamed),
                    syn::Fields::Unit => None,
                },
            },
        );
    }
    fn visit_vis_crate(&mut self, node: &syn::VisCrate) {
        return self.set_help(node, HelpItem::VisCrate);
    }
    fn visit_vis_public(&mut self, node: &syn::VisPublic) {
        return self.set_help(node, HelpItem::VisPublic);
    }
    fn visit_vis_restricted_first_pass(&mut self, node: &syn::VisRestricted) {
        let path = match &*node.path {
            path if path.is_ident("self") => VisRestrictedPath::Self_,
            path if path.is_ident("super") => VisRestrictedPath::Super,
            path if path.is_ident("crate") => VisRestrictedPath::Crate,
            _ => VisRestrictedPath::Path,
        };
        return self.set_help(
            node,
            HelpItem::VisRestricted {
                path,
                in_: node.in_token.is_some(),
            },
        );
    }
    fn visit_comment(&mut self, node: &Comment) {
        if node.doc.is_some() {
            return;
        }
        self.help = Some((node.range, HelpItem::Comment { block: node.block }));
    }
    fn visit_explicit_attribute(&mut self, node: &syn::Attribute) {
        let outer = outer_attr(node);

        return self.set_help(&node, HelpItem::Attribute { outer, known: None });
    }
}

fn outer_attr(attr: &syn::Attribute) -> bool {
    match attr.style {
        syn::AttrStyle::Outer => true,
        syn::AttrStyle::Inner(..) => false,
    }
}

fn special_path_help(
    analyzer: &mut NodeAnalyzer,
    leading_colon: Option<syn::token::Colon2>,
    leading_segment: Option<&syn::Ident>,
    can_be_receiver: bool,
) -> bool {
    if let Some(leading_colon) = leading_colon {
        if analyzer.within(leading_colon) {
            analyzer.set_help(&leading_colon, HelpItem::PathLeadingColon);
            return true;
        }
    }

    let mut settled = false;
    if let Some(ident) = leading_segment {
        if analyzer.within(&ident) {
            if ident == "super" {
                analyzer.set_help(&ident, HelpItem::PathSegmentSuper);
                settled = true;
            } else if ident == "self" {
                analyzer.set_help(
                    &ident,
                    if can_be_receiver {
                        let method = analyzer
                            .ancestors
                            .iter()
                            .rev()
                            .find_map(|(_, node)| match node {
                                Syn::ImplItemMethod(method) => Some(&method.sig),
                                Syn::TraitItemMethod(method) => Some(&method.sig),
                                _ => None,
                            })
                            .map(|sig| sig.ident.to_string());

                        HelpItem::ReceiverPath { method }
                    } else {
                        HelpItem::PathSegmentSelf
                    },
                );
                settled = true;
            } else if ident == "Self" {
                analyzer.set_help(&ident, HelpItem::PathSegmentSelfType);
                settled = true;
            } else if ident == "crate" {
                analyzer.set_help(&ident, HelpItem::PathSegmentCrate);
                settled = true;
            }
        }
    }

    settled
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

fn receiver_help(sig: &syn::Signature) -> Option<HelpItem> {
    let first = if let Some(arg) = sig.inputs.first() {
        arg
    } else {
        return None;
    };

    match first {
        syn::FnArg::Typed(pat_type) => {
            let pat_ident = match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident,
                _ => return None,
            };

            let is_self = pat_ident.by_ref.is_none()
                && pat_ident.subpat.is_none()
                && pat_ident.ident == "self";

            if !is_self {
                return None;
            }

            let mutability = pat_ident.mutability.is_some();

            match &*pat_type.ty {
                syn::Type::Path(type_path) if type_path.path.is_ident("Self") => {
                    Some(HelpItem::ValueSelf {
                        explicit: true,
                        mutability,
                    })
                }
                syn::Type::Reference(type_reference) => match &*type_reference.elem {
                    syn::Type::Path(type_ref_path) if type_ref_path.path.is_ident("Self") => {
                        if type_reference.mutability.is_some() {
                            Some(HelpItem::MutSelf {
                                explicit: true,
                                mutability,
                            })
                        } else {
                            Some(HelpItem::RefSelf {
                                explicit: true,
                                mutability,
                            })
                        }
                    }
                    _ => None,
                },
                _ => Some(HelpItem::SpecialSelf { mutability }),
            }
        }
        syn::FnArg::Receiver(receiver) => {
            let item = match (&receiver.reference, &receiver.mutability) {
                (Some(_), Some(_)) => HelpItem::MutSelf {
                    explicit: false,
                    mutability: false,
                },
                (Some(_), None) => HelpItem::RefSelf {
                    explicit: false,
                    mutability: false,
                },
                (None, mutability) => HelpItem::ValueSelf {
                    explicit: false,
                    mutability: mutability.is_some(),
                },
            };

            Some(item)
        }
    }
}

pub fn within_locations(loc: LineColumn, start: LineColumn, end: LineColumn) -> bool {
    (start.line < loc.line || (start.line == loc.line && start.column <= loc.column))
        && (loc.line < end.line || (loc.line == end.line && loc.column <= end.column))
}
