use crate::help::*;
use crate::ir::{Location, NodeId, Owner, Ptr, PtrData, Range};
use crate::syn_wrappers::{Comment, Syn, SynKind};
use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;

const DISTANCE_TYPE_PARAM_TO_CONTAINER: usize = 3;

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
    pub(crate) id_to_ptr: HashMap<NodeId, PtrData>,
    pub(crate) ptr_to_id: HashMap<Ptr, NodeId>,
    pub(crate) locations: Vec<(NodeId, Range)>,
    pub(crate) owner: Box<Owner>,
}

struct NodeAnalyzer<'a> {
    id: NodeId,
    location: Location,
    id_to_ptr: &'a HashMap<NodeId, PtrData>,
    ptr_to_id: &'a HashMap<Ptr, NodeId>,
    ancestors: &'a [(NodeId, Syn<'a>)],
    state: &'a mut HashMap<NodeId, NodeId>,
    help: Option<(Range, HelpItem)>,
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
            return Some(self.analyze_at_location(id, location, state.location_index, range));
        }

        for (idx, &(id, range)) in self.locations[state.location_index..].iter().enumerate() {
            if range.0 <= location && location <= range.1 {
                state.location_index += idx;
                return Some(self.analyze_at_location(id, location, state.location_index, range));
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
        let idx = self
            .locations
            .binary_search_by(|(_, range)| range.0.cmp(&location))
            .map(|idx| Some(idx))
            .unwrap_or_else(|err| if err == 0 { None } else { Some(err - 1) })?;

        let (id, range) = self.locations.get(idx).cloned()?;

        self.analyze_at_location(id, location, idx, range)
    }

    fn analyze_at_location(
        &self,
        id: NodeId,
        location: Location,
        idx: usize,
        range: Range,
    ) -> Option<AnalysisResult> {
        let mut candidates = [None; 3];
        candidates[0] = Some(id);

        if location == range.0 && idx > 0 {
            if let Some(&(prev_id, _)) = self
                .locations
                .get(idx - 1)
                .filter(|(_, prev_range)| prev_range.1 == range.0)
            {
                candidates[1] = Some(prev_id);
            }
        }

        if location == range.1 {
            if let Some(&(next_id, _)) = self
                .locations
                .get(idx + 1)
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
            .flat_map(|id| self.analyze_node_at_location(id, location))
            .next()
    }

    fn analyze_node_at_location(&self, id: NodeId, location: Location) -> Option<AnalysisResult> {
        let ancestors = {
            let mut ancestors = vec![];
            let mut id = id;

            while let Some(data) = self.id_to_ptr.get(&id) {
                ancestors.push((id, data.ptr.as_syn()));
                id = data.parent;
            }

            ancestors.reverse();

            ancestors
        };

        let mut state = HashMap::new();

        for (idx, &(node_id, node)) in ancestors.iter().enumerate() {
            if node_id == id {
                continue;
            }

            let mut node_analyzer = NodeAnalyzer::new(node_id, location, &self, &mut state);
            node_analyzer.ancestors = &ancestors[0..idx];

            node_analyzer.analyze_node_first_pass(node);

            if node_analyzer.help.is_some() {
                return node_analyzer.result();
            }
        }

        let mut ancestors = ancestors;
        while let Some((id, node)) = ancestors.pop() {
            let mut node_analyzer = NodeAnalyzer::new(id, location, &self, &mut state);
            node_analyzer.ancestors = &ancestors[..];
            node_analyzer.analyze_node(node);

            if node_analyzer.help.is_some() {
                return node_analyzer.result();
            }
        }

        None
    }

    fn descendant_of(&self, child: NodeId, ancestor: NodeId) -> Option<std::cmp::Ordering> {
        let mut node = child;
        let mut ancestors = HashSet::new();

        loop {
            node = if let Some(ptr) = self.id_to_ptr.get(&node) {
                ptr.parent
            } else {
                break;
            };

            ancestors.insert(node);

            if node == ancestor {
                return Some(std::cmp::Ordering::Less);
            }
        }

        node = ancestor;

        loop {
            node = if let Some(ptr) = self.id_to_ptr.get(&node) {
                ptr.parent
            } else {
                break;
            };

            if node == child {
                return Some(std::cmp::Ordering::Greater);
            } else if ancestors.contains(&node) {
                return None;
            }
        }

        None
    }
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

macro_rules! get_ancestor {
    ($self:ident, $ty:ident, $depth:expr) => {
        match $self.get_ancestor($depth) {
            Some(Syn::$ty(node)) => Some(node),
            _ => None,
        }
    };
}

impl<'a> NodeAnalyzer<'a> {
    fn new(
        id: NodeId,
        location: Location,
        analyzer: &'a Analyzer,
        state: &'a mut HashMap<NodeId, NodeId>,
    ) -> NodeAnalyzer<'a> {
        NodeAnalyzer {
            id,
            location,
            id_to_ptr: &analyzer.id_to_ptr,
            ptr_to_id: &analyzer.ptr_to_id,
            ancestors: &[],
            help: None,
            state,
        }
    }

    fn result(self) -> Option<AnalysisResult> {
        self.help
            .map(|((start, end), help)| AnalysisResult { start, end, help })
    }

    fn analyze_node_first_pass(&mut self, node: Syn) {
        match node {
            Syn::ExprForLoop(i) => self.visit_expr_for_loop_first_pass(i),
            Syn::Local(i) => self.visit_local_first_pass(i),
            Syn::TypeParam(i) => self.visit_type_param_first_pass(i),
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
            Syn::ExprBox(_i) => { /* self.visit_expr_box(i) */ }
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
            Syn::GenericParam(_i) => { /* self.visit_generic_param(i) */ }
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
            self.id_to_ptr
                .get(id)
                .into_iter()
                .flat_map(|data| data.children.iter())
                .filter_map(|child| self.id_to_ptr.get(child))
                .any(|child_data| child_data.ptr.kind() == SynKind::Comment)
        };

        if !node.path.is_ident("doc") || !is_comment(&self.id) {
            return self.visit_explicit_attribute(node);
        }

        let outer = outer_attr(node);

        let parent_data = self
            .id_to_ptr
            .get(&self.id)
            .map(|data| data.parent)
            .and_then(|parent_id| self.id_to_ptr.get(&parent_id));

        let parent = parent_data.map(|data| data.ptr.as_syn());

        let attributes = parent
            .as_ref()
            .and_then(|parent| parent.attributes())
            .unwrap_or(&[]);

        if attributes.len() == 0 {
            return;
        }

        let attribute_ids: Vec<NodeId> = {
            let mut ids = vec![];

            let node_to_id = parent_data
                .into_iter()
                .flat_map(|data| &data.children)
                .filter_map(|id| self.id_to_ptr.get(id).map(|data| (data, id)))
                .map(|(data, id)| (data.ptr.as_syn().data(), *id))
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
                    && is_comment(&attribute_ids[i])
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
                        && is_comment(&attribute_ids[i]))
            })
            .next()
            .map(|(i, _)| &attributes[i - 1])
            .unwrap_or(&attributes[0]);

        return self.set_help_between(start.span(), last.span(), HelpItem::DocBlock { outer });
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
    fn visit_const_param(&mut self, node: &syn::ConstParam) {
        token![self, node.const_token, ConstParam];
    }
    // EXAMPLE
    // impl<P: Deref<Target: Eq>> Eq for Pin<P> {}
    fn visit_expr_array(&mut self, node: &syn::ExprArray) {
        if !self.has_ancestor(2, SynKind::ExprReference) {
            self.set_help(node, HelpItem::ExprArray)
        }
    }
    fn visit_expr_assign(&mut self, node: &syn::ExprAssign) {
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssign);
    }
    fn visit_expr_assign_op(&mut self, node: &syn::ExprAssignOp) {
        // TODO: add context on the lvalue/place expression: deref, index, field access, etc.
        return self.set_help(node, HelpItem::ExprAssignOp);
    }
    fn visit_expr_async(&mut self, node: &syn::ExprAsync) {
        token![self, some node.capture, ExprAsyncMove];
        return self.set_help(node, HelpItem::ExprAsync);
    }
    fn visit_expr_await(&mut self, node: &syn::ExprAwait) {
        token![self, node.await_token, ExprAwait];
    }
    fn visit_expr_break(&mut self, node: &syn::ExprBreak) {
        return self.set_help(
            &node,
            HelpItem::ExprBreak {
                expr: node.expr.is_some(),
                label: node.label.as_ref().map(|l| l.to_string()),
            },
        );
    }
    fn visit_expr_cast(&mut self, node: &syn::ExprCast) {
        token![self, node.as_token, AsCast];
    }
    fn visit_expr_closure(&mut self, node: &syn::ExprClosure) {
        token![self, node.or1_token, ExprClosureArguments];
        token![self, node.or2_token, ExprClosureArguments];
        token![self, some node.asyncness, ExprClosureAsync];
        token![self, some node.capture, ExprClosureMove];
        token![self, some node.movability, ExprClosureStatic];
        return self.set_help(node, HelpItem::ExprClosure);
    }
    fn visit_expr_continue(&mut self, node: &syn::ExprContinue) {
        self.set_help(
            node,
            HelpItem::ExprContinue {
                label: node.label.as_ref().map(|l| l.to_string()),
            },
        );
    }
    fn visit_expr_field(&mut self, node: &syn::ExprField) {
        if let syn::Member::Unnamed(..) = node.member {
            return self.set_help_between(
                node.dot_token.span(),
                node.member.span(),
                HelpItem::ExprUnnamedField,
            );
        }
    }
    fn visit_expr_for_loop_first_pass(&mut self, node: &syn::ExprForLoop) {
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
    fn visit_expr_if(&mut self, node: &syn::ExprIf) {
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
    fn visit_expr_index(&mut self, node: &syn::ExprIndex) {
        let range = if let syn::Expr::Range(..) = &*node.index {
            true
        } else {
            false
        };

        return self.set_help(node, HelpItem::ExprIndex { range });
    }
    fn visit_expr_loop(&mut self, node: &syn::ExprLoop) {
        token![self, node.loop_token, ExprLoopToken];
    }
    fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
        token![self, node.match_token, ExprMatchToken];
    }
    fn visit_expr_range(&mut self, node: &syn::ExprRange) {
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
    fn visit_expr_reference(&mut self, node: &syn::ExprReference) {
        let item = if let syn::Expr::Array(_) = &*node.expr {
            HelpItem::ExprArraySlice
        } else {
            HelpItem::ExprReference {
                mutable: node.mutability.is_some(),
            }
        };

        return self.set_help(node, item);
    }
    fn visit_expr_repeat(&mut self, node: &syn::ExprRepeat) {
        return self.set_help(
            node,
            HelpItem::ExprRepeat {
                len: (&*node.len).to_token_stream().to_string(),
            },
        );
    }
    fn visit_expr_return(&mut self, node: &syn::ExprReturn) {
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
    fn visit_expr_struct(&mut self, node: &syn::ExprStruct) {
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
    }
    fn visit_expr_try(&mut self, node: &syn::ExprTry) {
        token![self, node.question_token, ExprTryQuestionMark];
    }
    fn visit_expr_try_block(&mut self, node: &syn::ExprTryBlock) {
        token![self, node.try_token, ExprTryBlock];
    }
    fn visit_expr_tuple(&mut self, node: &syn::ExprTuple) {
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
    fn visit_expr_type(&mut self, node: &syn::ExprType) {
        return self.set_help(node, HelpItem::ExprType);
    }
    fn visit_expr_unsafe(&mut self, node: &syn::ExprUnsafe) {
        token![self, node.unsafe_token, ExprUnsafe];
    }
    fn visit_expr_while(&mut self, node: &syn::ExprWhile) {
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
            None => token![self, node.while_token, ExprWhile],
        }
    }
    fn visit_expr_yield(&mut self, node: &syn::ExprYield) {
        token![self, node.yield_token, ExprYield];
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

        if let (None, syn::Pat::Ident(syn::PatIdent { ident, .. })) = (node.colon_token, &*node.pat)
        {
            return self.set_help(
                node,
                HelpItem::FieldPatShorthand {
                    ident: ident.to_string(),
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
    fn visit_impl_item_const(&mut self, node: &syn::ImplItemConst) {
        token![self, node.const_token, ImplItemConst];
    }
    fn visit_impl_item_method(&mut self, node: &syn::ImplItemMethod) {
        if !self.between(&node.sig.fn_token, &node.sig.ident) {
            return;
        }

        let is_method = receiver_help(&node.sig).is_some();
        let trait_ = get_ancestor![self, ItemImpl, 2].and_then(|item| item.trait_.as_ref());

        let of = if is_method {
            FnOf::Method
        } else {
            FnOf::AssociatedFunction
        };

        if let Some(impl_) = get_ancestor![self, ItemImpl, 2] {
            // TODO: handle bang
            // TODO: better formatting
            let trait_ = trait_.map(|(_bang, path, _)| path.to_token_stream().to_string());
            let self_ty = (&*impl_.self_ty).to_token_stream().to_string();
            return self.set_help_between(
                node.sig.fn_token.span(),
                node.sig.ident.span(),
                HelpItem::ImplItemMethod {
                    of,
                    trait_,
                    self_ty,
                },
            );
        }
    }
    fn visit_impl_item_type(&mut self, node: &syn::ImplItemType) {
        token![self, node.type_token, ImplItemType];
    }
    fn visit_item_const(&mut self, node: &syn::ItemConst) {
        token![self, node.const_token, ItemConst];
    }
    fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        token![self, node.enum_token => node.ident, * HelpItem::ItemEnum {
            empty: node.variants.is_empty()
        }];
    }
    fn visit_item_extern_crate(&mut self, node: &syn::ItemExternCrate) {
        if let Some((as_token, _)) = node.rename {
            token![self, as_token, AsRenameExternCrate];
        }
        token![self, node.extern_token, ItemExternCrate];
    }
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        token![self, node.sig.fn_token => node.sig.ident, ItemFn];
    }
    fn visit_item_foreign_mod(&mut self, node: &syn::ItemForeignMod) {
        token![self, node.abi, ItemForeignModAbi];
    }
    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        token![self, some node.unsafety, ItemUnsafeImpl];
        if let Some((_, _, for_token)) = node.trait_ {
            token![self, for_token, ItemImplForTrait];
        }
        token![
            self,
            node.impl_token,
            *HelpItem::ItemImpl {
                trait_: node.trait_.is_some(),
                negative: node
                    .trait_
                    .as_ref()
                    .and_then(|(bang, _, _)| bang.as_ref())
                    .is_some()
            }
        ];
    }
    fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
        if let Some(ident) = &node.ident {
            if node.mac.path.is_ident("macro_rules") {
                return self.set_help(
                    node,
                    HelpItem::ItemMacroRules {
                        name: ident.to_string(),
                    },
                );
            }
        }
    }
    fn visit_item_mod(&mut self, node: &syn::ItemMod) {
        if node.content.is_some() {
            if self.between(&node.mod_token, &node.ident) {
                return self.set_help_between(
                    node.mod_token.span(),
                    node.ident.span(),
                    HelpItem::ItemInlineMod,
                );
            }
        } else {
            return self.set_help(&node, HelpItem::ItemExternMod);
        }
    }
    fn visit_item_static(&mut self, node: &syn::ItemStatic) {
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
    fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
        let unit = match node.fields {
            syn::Fields::Unit => true,
            _ => false,
        };
        if self.between_spans(node.struct_token.span(), node.ident.span()) {
            return self.set_help_between(
                node.struct_token.span(),
                node.ident.span(),
                HelpItem::ItemStruct {
                    unit,
                    name: node.ident.to_string(),
                },
            );
        }
    }
    fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        token![self, some node.unsafety, ItemUnsafeTrait];
        token![self, some node.auto_token, ItemAutoTrait];
        token![self, node.trait_token, ItemTrait];
        if let Some(colon_token) = node.colon_token {
            if self.within(colon_token) {
                let last = node
                    .supertraits
                    .last()
                    .map(|t| t.span())
                    .unwrap_or(colon_token.span());
                return self.set_help_between(
                    colon_token.span(),
                    last,
                    HelpItem::ItemTraitSupertraits,
                );
            }
        }
    }
    fn visit_item_trait_alias(&mut self, node: &syn::ItemTraitAlias) {
        token![self, node.trait_token, ItemTraitAlias];
    }
    fn visit_item_type(&mut self, node: &syn::ItemType) {
        token![self, node.type_token => node.ident, ItemType];
    }
    fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        token![self, node.union_token, ItemUnion];
    }
    fn visit_item_use(&mut self, node: &syn::ItemUse) {
        token![self, some node.leading_colon, PathLeadingColon];
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemUse);
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
    fn visit_pat_box(&mut self, node: &syn::PatBox) {
        token![self, node.box_token, PatBox];
    }
    fn visit_pat_ident(&mut self, node: &syn::PatIdent) {
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
    fn visit_pat_or(&mut self, node: &syn::PatOr) {
        token![self, some node.leading_vert, PatOrLeading];
        for pair in node.cases.pairs() {
            token![self, some pair.punct(), PatOr];
        }
    }
    fn visit_pat_range(&mut self, node: &syn::PatRange) {
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
    fn visit_pat_struct(&mut self, node: &syn::PatStruct) {
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
    fn visit_pat_tuple(&mut self, node: &syn::PatTuple) {
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
    fn visit_pat_tuple_struct(&mut self, node: &syn::PatTupleStruct) {
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
    fn visit_pat_wild(&mut self, node: &syn::PatWild) {
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
    fn visit_path(&mut self, node: &syn::Path) {
        let qself = loop {
            let ancestor = if let Some(ancestor) = self
                .id_to_ptr
                .get(&self.id)
                .and_then(|data| self.id_to_ptr.get(&data.parent))
            {
                ancestor.ptr.as_syn()
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
    fn visit_predicate_type(&mut self, node: &syn::PredicateType) {
        token![self, node.lifetimes, BoundLifetimes];
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
            if node.as_token.is_some() {
                HelpItem::QSelfAsTrait
            } else {
                HelpItem::QSelf
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
                    lifetime: format!("{}", lifetime.lifetime),
                    multiple,
                    ty: format!("{}", node.path.to_token_stream()),
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
    fn visit_trait_item_const(&mut self, node: &syn::TraitItemConst) {
        token![self, node.const_token, TraitItemConst];
    }
    fn visit_trait_item_method(&mut self, node: &syn::TraitItemMethod) {
        let of = if receiver_help(&node.sig).is_some() {
            FnOf::Method
        } else {
            FnOf::AssociatedFunction
        };
        if !self.between(&node.sig.fn_token, &node.sig.ident) {
            return;
        }
        if let Some(trait_) = get_ancestor![self, ItemTrait, 2] {
            let trait_ = trait_.ident.to_string();
            return self.set_help_between(
                node.sig.fn_token.span(),
                node.sig.ident.span(),
                HelpItem::TraitItemMethod {
                    of,
                    default: node.default.is_some(),
                    trait_,
                },
            );
        }
    }
    fn visit_trait_item_type(&mut self, node: &syn::TraitItemType) {
        token![self, node.type_token, TraitItemType];
    }
    fn visit_type_array(&mut self, node: &syn::TypeArray) {
        if self.between_locations(node.span().start(), node.elem.span().start())
            || self.between_spans(node.semi_token.span(), node.span())
        {
            return self.set_help(node, HelpItem::TypeArray);
        }
    }
    fn visit_type_bare_fn(&mut self, node: &syn::TypeBareFn) {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        token![self, some node.lifetimes, BoundLifetimesBareFnType];
        return self.set_help(node, HelpItem::TypeBareFn);
    }
    fn visit_type_impl_trait(&mut self, node: &syn::TypeImplTrait) {
        token![self, node.impl_token, TypeImplTrait];
    }
    fn visit_type_infer(&mut self, node: &syn::TypeInfer) {
        return self.set_help(node, HelpItem::TypeInfer);
    }
    fn visit_type_never(&mut self, node: &syn::TypeNever) {
        return self.set_help(node, HelpItem::TypeNever);
    }
    fn visit_type_param(&mut self, node: &syn::TypeParam) {
        let is_def = self
            .get_ancestor(DISTANCE_TYPE_PARAM_TO_CONTAINER)
            .and_then(|container| self.ptr_to_id.get(&Ptr::new(container.clone())))
            .map(|container_id| Some(container_id) == self.state.get(&self.id))
            .unwrap_or(false);

        if is_def {
            return self.set_help(
                &node.ident,
                HelpItem::TypeParam {
                    name: node.ident.to_token_stream().to_string(),
                },
            );
        }
    }
    fn visit_type_path(&mut self, node: &syn::TypePath) {
        if let Some(item) = well_known_type(node) {
            // SHORTCUT
            if let HelpItem::KnownTypeStr = item {
                if self.has_ancestor(2, SynKind::TypeReference) {
                    return;
                }
            }
            return self.set_help(node, item);
        }
    }
    fn visit_type_ptr(&mut self, node: &syn::TypePtr) {
        let end = node
            .const_token
            .map(|t| t.span())
            .or_else(|| node.mutability.map(|t| t.span()));

        if let Some(end) = end {
            return self.set_help_between(
                node.span(),
                end,
                if node.const_token.is_some() {
                    HelpItem::TypeConstPtr
                } else {
                    HelpItem::TypeMutPtr
                },
            );
        }
    }
    fn visit_type_reference(&mut self, node: &syn::TypeReference) {
        if let syn::Type::Path(type_path) = &*node.elem {
            if let Some(HelpItem::KnownTypeStr) = well_known_type(type_path) {
                return self.set_help(
                    node,
                    HelpItem::KnownTypeStrSlice {
                        mutability: node.mutability.is_some(),
                    },
                );
            }
        }
        let last_span = node
            .mutability
            .map(|t| t.span())
            .or_else(|| node.lifetime.as_ref().map(|t| t.span()))
            .unwrap_or_else(|| node.and_token.span());

        if self.between_spans(node.and_token.span(), last_span) {
            return self.set_help(
                &node,
                HelpItem::TypeReference {
                    lifetime: node.lifetime.is_some(),
                    mutable: node.mutability.is_some(),
                    ty: format!("{}", node.elem.to_token_stream()),
                },
            );
        }
    }
    fn visit_type_slice(&mut self, node: &syn::TypeSlice) {
        let (dynamic, start) = if let Some(type_ref) = get_ancestor![self, TypeReference, 2] {
            (true, type_ref.span())
        } else {
            (false, node.span())
        };
        return self.set_help_between(
            start,
            node.span(),
            HelpItem::TypeSlice {
                dynamic,
                ty: node.elem.to_token_stream().to_string(),
            },
        );
    }
    // Fun fact: a legacy trait object without `dyn` can probably only be recognized
    // by compiling the code.
    fn visit_type_trait_object(&mut self, node: &syn::TypeTraitObject) {
        if let Some(plus_token) = node
            .bounds
            .pairs()
            .filter_map(|pair| pair.punct().cloned())
            .find(|punct| self.within(punct))
        {
            return self.set_help(&plus_token, HelpItem::TypeParamBoundAdd);
        }

        let ty = if let Some(syn::TypeParamBound::Trait(trait_bound)) = node.bounds.first() {
            trait_bound
        } else {
            return;
        };

        let lifetime = node
            .bounds
            .iter()
            .filter_map(|bound| match bound {
                syn::TypeParamBound::Lifetime(lifetime) => Some(lifetime),
                _ => None,
            })
            .next();

        let multiple = node
            .bounds
            .iter()
            .filter(|bound| match bound {
                syn::TypeParamBound::Trait(..) => true,
                _ => false,
            })
            .skip(1)
            .next()
            .is_some();

        return self.set_help(
            node,
            HelpItem::TypeTraitObject {
                lifetime: lifetime.map(|lt| format!("{}", lt)),
                multiple,
                dyn_: node.dyn_token.is_some(),
                ty: format!("{}", ty.path.to_token_stream()),
            },
        );
    }
    fn visit_type_tuple(&mut self, node: &syn::TypeTuple) {
        if node.elems.is_empty() {
            return self.set_help(node, HelpItem::TypeTupleUnit);
        }
        return self.set_help(
            node,
            HelpItem::TypeTuple {
                single_comma: node.elems.len() == 1 && node.elems.trailing_punct(),
            },
        );
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
    fn visit_type_param_first_pass(&mut self, _: &syn::TypeParam) {
        let scope_id = match self.get_ancestor(DISTANCE_TYPE_PARAM_TO_CONTAINER) {
            Some(i @ Syn::ItemStruct(_)) => self.ptr_to_id.get(&Ptr::new(i.clone())).cloned(),
            _ => None,
        };
        if let Some(scope_id) = scope_id {
            self.state.insert(self.id, scope_id);
        }
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
    fn visit_where_clause(&mut self, node: &syn::WhereClause) {
        token![self, node.where_token, WhereClause];
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
