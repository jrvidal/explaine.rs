use crate::help::HelpItem;
use crate::ir::{Location, Ptr, PtrData};
use proc_macro2::{LineColumn, Span};
use std::collections::HashMap;

pub struct Analyzer {
    pub(crate) id_to_ptr: HashMap<usize, PtrData>,
    pub(crate) ptr_to_id: HashMap<Ptr, usize>,
    pub(crate) locations: Vec<(usize, Location)>,
}

macro_rules! analyze {
    ($self:ident, $node:ident, $ty:path, $method:ident) => {
        if let Some(node) = $node.downcast::<$ty>() {
            println!("downcasted to {}", stringify!($ty));
            $self.$method(node);
            return;
        }
    };
}

struct NodeAnalyzer<'a> {
    location: Location,
    id_to_ptr: &'a HashMap<usize, PtrData>,
    help: Option<((Location, Location), HelpItem)>,
}

pub struct AnalysisResult {
    pub start: Location,
    pub end: Location,
    pub help: HelpItem,
}

impl Analyzer {
    pub fn analyze_item(&self, item: usize) -> Option<Option<AnalysisResult>> {
        self.locations.get(item).map(|(_, loc)| self.analyze(*loc))
    }

    pub fn analyze(&self, location: Location) -> Option<AnalysisResult> {
        let result = self
            .locations
            .binary_search_by(|(_, loc)| loc.cmp(&location))
            .unwrap_or_else(|err| err);
        println!("index of location = {}", result);
        let id = if let Some((id, _)) = self.locations.get(result) {
            *id
        } else {
            return None;
        };

        println!("id of element = {}", id);

        let mut node = if let Some(node) = self.id_to_ptr.get(&id) {
            node
        } else {
            return None;
        };

        let mut node_analyzer = NodeAnalyzer {
            location,
            id_to_ptr: &self.id_to_ptr,
            help: None,
        };

        loop {
            node_analyzer.analyze_node(&node.ptr);

            if let Some(((start, end), help)) = node_analyzer.help {
                break Some(AnalysisResult { start, end, help });
            }

            if let Some(parent) = self.id_to_ptr.get(&node.parent) {
                node = parent;
            } else {
                break None;
            }
        }
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

impl<'a> NodeAnalyzer<'a> {
    fn analyze_node(&mut self, node: &Ptr) {
        analyze![self, node, syn::Abi, visit_abi];
        analyze![
            self,
            node,
            syn::AngleBracketedGenericArguments,
            visit_angle_bracketed_generic_arguments
        ];
        analyze![self, node, syn::Arm, visit_arm];
        analyze![self, node, syn::AttrStyle, visit_attr_style];
        analyze![self, node, syn::Attribute, visit_attribute];
        analyze![self, node, syn::BareFnArg, visit_bare_fn_arg];
        analyze![self, node, syn::BinOp, visit_bin_op];
        analyze![self, node, syn::Binding, visit_binding];
        analyze![self, node, syn::Block, visit_block];
        analyze![self, node, syn::BoundLifetimes, visit_bound_lifetimes];
        analyze![self, node, syn::ConstParam, visit_const_param];
        analyze![self, node, syn::Constraint, visit_constraint];
        analyze![self, node, syn::Expr, visit_expr];
        analyze![self, node, syn::ExprArray, visit_expr_array];
        analyze![self, node, syn::ExprAssign, visit_expr_assign];
        analyze![self, node, syn::ExprAssignOp, visit_expr_assign_op];
        analyze![self, node, syn::ExprAsync, visit_expr_async];
        analyze![self, node, syn::ExprAwait, visit_expr_await];
        analyze![self, node, syn::ExprBinary, visit_expr_binary];
        analyze![self, node, syn::ExprBlock, visit_expr_block];
        analyze![self, node, syn::ExprBox, visit_expr_box];
        analyze![self, node, syn::ExprBreak, visit_expr_break];
        analyze![self, node, syn::ExprCall, visit_expr_call];
        analyze![self, node, syn::ExprCast, visit_expr_cast];
        analyze![self, node, syn::ExprClosure, visit_expr_closure];
        analyze![self, node, syn::ExprContinue, visit_expr_continue];
        analyze![self, node, syn::ExprField, visit_expr_field];
        analyze![self, node, syn::ExprForLoop, visit_expr_for_loop];
        analyze![self, node, syn::ExprGroup, visit_expr_group];
        analyze![self, node, syn::ExprIf, visit_expr_if];
        analyze![self, node, syn::ExprIndex, visit_expr_index];
        analyze![self, node, syn::ExprLet, visit_expr_let];
        analyze![self, node, syn::ExprLit, visit_expr_lit];
        analyze![self, node, syn::ExprLoop, visit_expr_loop];
        analyze![self, node, syn::ExprMacro, visit_expr_macro];
        analyze![self, node, syn::ExprMatch, visit_expr_match];
        analyze![self, node, syn::ExprMethodCall, visit_expr_method_call];
        analyze![self, node, syn::ExprParen, visit_expr_paren];
        analyze![self, node, syn::ExprPath, visit_expr_path];
        analyze![self, node, syn::ExprRange, visit_expr_range];
        analyze![self, node, syn::ExprReference, visit_expr_reference];
        analyze![self, node, syn::ExprRepeat, visit_expr_repeat];
        analyze![self, node, syn::ExprReturn, visit_expr_return];
        analyze![self, node, syn::ExprStruct, visit_expr_struct];
        analyze![self, node, syn::ExprTry, visit_expr_try];
        analyze![self, node, syn::ExprTryBlock, visit_expr_try_block];
        analyze![self, node, syn::ExprTuple, visit_expr_tuple];
        analyze![self, node, syn::ExprType, visit_expr_type];
        analyze![self, node, syn::ExprUnary, visit_expr_unary];
        analyze![self, node, syn::ExprUnsafe, visit_expr_unsafe];
        analyze![self, node, syn::ExprWhile, visit_expr_while];
        analyze![self, node, syn::ExprYield, visit_expr_yield];
        analyze![self, node, syn::Field, visit_field];
        analyze![self, node, syn::FieldPat, visit_field_pat];
        analyze![self, node, syn::FieldValue, visit_field_value];
        analyze![self, node, syn::Fields, visit_fields];
        analyze![self, node, syn::FieldsNamed, visit_fields_named];
        analyze![self, node, syn::FieldsUnnamed, visit_fields_unnamed];
        analyze![self, node, syn::File, visit_file];
        analyze![self, node, syn::FnArg, visit_fn_arg];
        analyze![self, node, syn::ForeignItem, visit_foreign_item];
        analyze![self, node, syn::ForeignItemFn, visit_foreign_item_fn];
        analyze![self, node, syn::ForeignItemMacro, visit_foreign_item_macro];
        analyze![
            self,
            node,
            syn::ForeignItemStatic,
            visit_foreign_item_static
        ];
        analyze![self, node, syn::ForeignItemType, visit_foreign_item_type];
        analyze![self, node, syn::GenericArgument, visit_generic_argument];
        analyze![
            self,
            node,
            syn::GenericMethodArgument,
            visit_generic_method_argument
        ];
        analyze![self, node, syn::GenericParam, visit_generic_param];
        analyze![self, node, syn::Generics, visit_generics];
        analyze![self, node, proc_macro2::Ident, visit_ident];
        analyze![self, node, syn::ImplItem, visit_impl_item];
        analyze![self, node, syn::ImplItemConst, visit_impl_item_const];
        analyze![self, node, syn::ImplItemMacro, visit_impl_item_macro];
        analyze![self, node, syn::ImplItemMethod, visit_impl_item_method];
        analyze![self, node, syn::ImplItemType, visit_impl_item_type];
        analyze![self, node, syn::Index, visit_index];
        analyze![self, node, syn::Item, visit_item];
        analyze![self, node, syn::ItemConst, visit_item_const];
        analyze![self, node, syn::ItemEnum, visit_item_enum];
        analyze![self, node, syn::ItemExternCrate, visit_item_extern_crate];
        analyze![self, node, syn::ItemFn, visit_item_fn];
        analyze![self, node, syn::ItemForeignMod, visit_item_foreign_mod];
        analyze![self, node, syn::ItemImpl, visit_item_impl];
        analyze![self, node, syn::ItemMacro, visit_item_macro];
        analyze![self, node, syn::ItemMacro2, visit_item_macro2];
        analyze![self, node, syn::ItemMod, visit_item_mod];
        analyze![self, node, syn::ItemStatic, visit_item_static];
        analyze![self, node, syn::ItemStruct, visit_item_struct];
        analyze![self, node, syn::ItemTrait, visit_item_trait];
        analyze![self, node, syn::ItemTraitAlias, visit_item_trait_alias];
        analyze![self, node, syn::ItemType, visit_item_type];
        analyze![self, node, syn::ItemUnion, visit_item_union];
        analyze![self, node, syn::ItemUse, visit_item_use];
        analyze![self, node, syn::Label, visit_label];
        analyze![self, node, syn::Lifetime, visit_lifetime];
        analyze![self, node, syn::LifetimeDef, visit_lifetime_def];
        analyze![self, node, syn::Lit, visit_lit];
        analyze![self, node, syn::LitBool, visit_lit_bool];
        analyze![self, node, syn::LitByte, visit_lit_byte];
        analyze![self, node, syn::LitByteStr, visit_lit_byte_str];
        analyze![self, node, syn::LitChar, visit_lit_char];
        analyze![self, node, syn::LitFloat, visit_lit_float];
        analyze![self, node, syn::LitInt, visit_lit_int];
        analyze![self, node, syn::LitStr, visit_lit_str];
        analyze![self, node, syn::Local, visit_local];
        analyze![self, node, syn::Macro, visit_macro];
        analyze![self, node, syn::MacroDelimiter, visit_macro_delimiter];
        analyze![self, node, syn::Member, visit_member];
        analyze![self, node, syn::Meta, visit_meta];
        analyze![self, node, syn::MetaList, visit_meta_list];
        analyze![self, node, syn::MetaNameValue, visit_meta_name_value];
        analyze![self, node, syn::MethodTurbofish, visit_method_turbofish];
        analyze![self, node, syn::NestedMeta, visit_nested_meta];
        analyze![
            self,
            node,
            syn::ParenthesizedGenericArguments,
            visit_parenthesized_generic_arguments
        ];
        analyze![self, node, syn::Pat, visit_pat];
        analyze![self, node, syn::PatBox, visit_pat_box];
        analyze![self, node, syn::PatIdent, visit_pat_ident];
        analyze![self, node, syn::PatLit, visit_pat_lit];
        analyze![self, node, syn::PatMacro, visit_pat_macro];
        analyze![self, node, syn::PatOr, visit_pat_or];
        analyze![self, node, syn::PatPath, visit_pat_path];
        analyze![self, node, syn::PatRange, visit_pat_range];
        analyze![self, node, syn::PatReference, visit_pat_reference];
        analyze![self, node, syn::PatRest, visit_pat_rest];
        analyze![self, node, syn::PatSlice, visit_pat_slice];
        analyze![self, node, syn::PatStruct, visit_pat_struct];
        analyze![self, node, syn::PatTuple, visit_pat_tuple];
        analyze![self, node, syn::PatTupleStruct, visit_pat_tuple_struct];
        analyze![self, node, syn::PatType, visit_pat_type];
        analyze![self, node, syn::PatWild, visit_pat_wild];
        analyze![self, node, syn::Path, visit_path];
        analyze![self, node, syn::PathArguments, visit_path_arguments];
        analyze![self, node, syn::PathSegment, visit_path_segment];
        analyze![self, node, syn::PredicateEq, visit_predicate_eq];
        analyze![self, node, syn::PredicateLifetime, visit_predicate_lifetime];
        analyze![self, node, syn::PredicateType, visit_predicate_type];
        analyze![self, node, syn::QSelf, visit_qself];
        analyze![self, node, syn::RangeLimits, visit_range_limits];
        analyze![self, node, syn::Receiver, visit_receiver];
        analyze![self, node, syn::ReturnType, visit_return_type];
        analyze![self, node, syn::Signature, visit_signature];
        analyze![self, node, syn::Stmt, visit_stmt];
        analyze![self, node, syn::TraitBound, visit_trait_bound];
        analyze![
            self,
            node,
            syn::TraitBoundModifier,
            visit_trait_bound_modifier
        ];
        analyze![self, node, syn::TraitItem, visit_trait_item];
        analyze![self, node, syn::TraitItemConst, visit_trait_item_const];
        analyze![self, node, syn::TraitItemMacro, visit_trait_item_macro];
        analyze![self, node, syn::TraitItemMethod, visit_trait_item_method];
        analyze![self, node, syn::TraitItemType, visit_trait_item_type];
        analyze![self, node, syn::Type, visit_type];
        analyze![self, node, syn::TypeArray, visit_type_array];
        analyze![self, node, syn::TypeBareFn, visit_type_bare_fn];
        analyze![self, node, syn::TypeGroup, visit_type_group];
        analyze![self, node, syn::TypeImplTrait, visit_type_impl_trait];
        analyze![self, node, syn::TypeInfer, visit_type_infer];
        analyze![self, node, syn::TypeMacro, visit_type_macro];
        analyze![self, node, syn::TypeNever, visit_type_never];
        analyze![self, node, syn::TypeParam, visit_type_param];
        analyze![self, node, syn::TypeParamBound, visit_type_param_bound];
        analyze![self, node, syn::TypeParen, visit_type_paren];
        analyze![self, node, syn::TypePath, visit_type_path];
        analyze![self, node, syn::TypePtr, visit_type_ptr];
        analyze![self, node, syn::TypeReference, visit_type_reference];
        analyze![self, node, syn::TypeSlice, visit_type_slice];
        analyze![self, node, syn::TypeTraitObject, visit_type_trait_object];
        analyze![self, node, syn::TypeTuple, visit_type_tuple];
        analyze![self, node, syn::UnOp, visit_un_op];
        analyze![self, node, syn::UseGlob, visit_use_glob];
        analyze![self, node, syn::UseGroup, visit_use_group];
        analyze![self, node, syn::UseName, visit_use_name];
        analyze![self, node, syn::UsePath, visit_use_path];
        analyze![self, node, syn::UseRename, visit_use_rename];
        analyze![self, node, syn::UseTree, visit_use_tree];
        analyze![self, node, syn::Variadic, visit_variadic];
        analyze![self, node, syn::Variant, visit_variant];
        analyze![self, node, syn::VisCrate, visit_vis_crate];
        analyze![self, node, syn::VisPublic, visit_vis_public];
        analyze![self, node, syn::VisRestricted, visit_vis_restricted];
        analyze![self, node, syn::Visibility, visit_visibility];
        analyze![self, node, syn::WhereClause, visit_where_clause];
        analyze![self, node, syn::WherePredicate, visit_where_predicate];
    }

    fn set_help<S: syn::spanned::Spanned>(&mut self, node: S, item: HelpItem) {
        let span = node.span();
        self.help = Some(((span.start().into(), span.end().into()), item));
    }

    fn within<S: syn::spanned::Spanned>(&self, item: S) -> bool {
        let span = item.span();
        self.between_spans(span, span)
    }
    fn between_spans(&self, start: Span, end: Span) -> bool {
        self.between_locations(start.start(), end.end())
    }

    fn between_locations(&self, start: LineColumn, end: LineColumn) -> bool {
        let loc = self.location;
        super::within_locations(
            LineColumn {
                line: loc.line,
                column: loc.column,
            },
            start,
            end,
        )
    }

    fn visit_abi(&mut self, _: &syn::Abi) {}

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
    fn visit_attr_style(&mut self, _: &syn::AttrStyle) {}
    fn visit_attribute(&mut self, _: &syn::Attribute) {}
    fn visit_bare_fn_arg(&mut self, _: &syn::BareFnArg) {}
    fn visit_bin_op(&mut self, _: &syn::BinOp) {}
    fn visit_binding(&mut self, _: &syn::Binding) {}
    fn visit_block(&mut self, _: &syn::Block) {}
    fn visit_bound_lifetimes(&mut self, _: &syn::BoundLifetimes) {}
    fn visit_const_param(&mut self, _: &syn::ConstParam) {}
    fn visit_constraint(&mut self, _: &syn::Constraint) {}
    fn visit_expr(&mut self, _: &syn::Expr) {}
    fn visit_expr_array(&mut self, _: &syn::ExprArray) {}
    fn visit_expr_assign(&mut self, _: &syn::ExprAssign) {}
    fn visit_expr_assign_op(&mut self, _: &syn::ExprAssignOp) {}
    fn visit_expr_async(&mut self, _: &syn::ExprAsync) {}
    fn visit_expr_await(&mut self, _: &syn::ExprAwait) {}
    fn visit_expr_binary(&mut self, _: &syn::ExprBinary) {}
    fn visit_expr_block(&mut self, _: &syn::ExprBlock) {}
    fn visit_expr_box(&mut self, _: &syn::ExprBox) {}
    fn visit_expr_break(&mut self, _: &syn::ExprBreak) {}
    fn visit_expr_call(&mut self, _: &syn::ExprCall) {}
    fn visit_expr_cast(&mut self, _: &syn::ExprCast) {}
    fn visit_expr_closure(&mut self, _: &syn::ExprClosure) {}
    fn visit_expr_continue(&mut self, _: &syn::ExprContinue) {}
    fn visit_expr_field(&mut self, _: &syn::ExprField) {}
    fn visit_expr_for_loop(&mut self, _: &syn::ExprForLoop) {}
    fn visit_expr_group(&mut self, _: &syn::ExprGroup) {}
    fn visit_expr_if(&mut self, _: &syn::ExprIf) {}
    fn visit_expr_index(&mut self, _: &syn::ExprIndex) {}
    fn visit_expr_let(&mut self, _: &syn::ExprLet) {}
    fn visit_expr_lit(&mut self, _: &syn::ExprLit) {}
    fn visit_expr_loop(&mut self, _: &syn::ExprLoop) {}
    fn visit_expr_macro(&mut self, _: &syn::ExprMacro) {}
    fn visit_expr_match(&mut self, _: &syn::ExprMatch) {}
    fn visit_expr_method_call(&mut self, _: &syn::ExprMethodCall) {}
    fn visit_expr_paren(&mut self, _: &syn::ExprParen) {}
    fn visit_expr_path(&mut self, _: &syn::ExprPath) {}
    fn visit_expr_range(&mut self, _: &syn::ExprRange) {}
    fn visit_expr_reference(&mut self, _: &syn::ExprReference) {}
    fn visit_expr_repeat(&mut self, _: &syn::ExprRepeat) {}
    fn visit_expr_return(&mut self, _: &syn::ExprReturn) {}
    fn visit_expr_struct(&mut self, _: &syn::ExprStruct) {}
    fn visit_expr_try(&mut self, _: &syn::ExprTry) {}
    fn visit_expr_try_block(&mut self, _: &syn::ExprTryBlock) {}
    fn visit_expr_tuple(&mut self, _: &syn::ExprTuple) {}
    fn visit_expr_type(&mut self, _: &syn::ExprType) {}
    fn visit_expr_unary(&mut self, _: &syn::ExprUnary) {}
    fn visit_expr_unsafe(&mut self, _: &syn::ExprUnsafe) {}
    fn visit_expr_while(&mut self, _: &syn::ExprWhile) {}
    fn visit_expr_yield(&mut self, _: &syn::ExprYield) {}
    fn visit_field(&mut self, _: &syn::Field) {}
    fn visit_field_pat(&mut self, _: &syn::FieldPat) {}
    fn visit_field_value(&mut self, _: &syn::FieldValue) {}
    fn visit_fields(&mut self, _: &syn::Fields) {}
    fn visit_fields_named(&mut self, _: &syn::FieldsNamed) {}
    fn visit_fields_unnamed(&mut self, _: &syn::FieldsUnnamed) {}
    fn visit_file(&mut self, _: &syn::File) {}
    fn visit_fn_arg(&mut self, _: &syn::FnArg) {}
    fn visit_foreign_item(&mut self, _: &syn::ForeignItem) {}
    fn visit_foreign_item_fn(&mut self, _: &syn::ForeignItemFn) {}
    fn visit_foreign_item_macro(&mut self, _: &syn::ForeignItemMacro) {}
    fn visit_foreign_item_static(&mut self, _: &syn::ForeignItemStatic) {}
    fn visit_foreign_item_type(&mut self, _: &syn::ForeignItemType) {}
    fn visit_generic_argument(&mut self, _: &syn::GenericArgument) {}
    fn visit_generic_method_argument(&mut self, _: &syn::GenericMethodArgument) {}
    fn visit_generic_param(&mut self, _: &syn::GenericParam) {}
    fn visit_generics(&mut self, _: &syn::Generics) {}
    fn visit_ident(&mut self, _: &proc_macro2::Ident) {}
    fn visit_impl_item(&mut self, _: &syn::ImplItem) {}
    fn visit_impl_item_const(&mut self, _: &syn::ImplItemConst) {}
    fn visit_impl_item_macro(&mut self, _: &syn::ImplItemMacro) {}
    fn visit_impl_item_method(&mut self, _: &syn::ImplItemMethod) {}
    fn visit_impl_item_type(&mut self, _: &syn::ImplItemType) {}
    fn visit_index(&mut self, _: &syn::Index) {}
    fn visit_item(&mut self, _: &syn::Item) {}
    fn visit_item_const(&mut self, _: &syn::ItemConst) {}
    fn visit_item_enum(&mut self, _: &syn::ItemEnum) {}
    fn visit_item_extern_crate(&mut self, _: &syn::ItemExternCrate) {}
    fn visit_item_fn(&mut self, _: &syn::ItemFn) {}
    fn visit_item_foreign_mod(&mut self, _: &syn::ItemForeignMod) {}
    fn visit_item_impl(&mut self, _: &syn::ItemImpl) {}
    fn visit_item_macro(&mut self, _: &syn::ItemMacro) {}
    fn visit_item_macro2(&mut self, _: &syn::ItemMacro2) {}
    fn visit_item_mod(&mut self, _: &syn::ItemMod) {}
    fn visit_item_static(&mut self, _: &syn::ItemStatic) {}
    fn visit_item_struct(&mut self, _: &syn::ItemStruct) {}
    fn visit_item_trait(&mut self, _: &syn::ItemTrait) {}
    fn visit_item_trait_alias(&mut self, _: &syn::ItemTraitAlias) {}
    fn visit_item_type(&mut self, _: &syn::ItemType) {}
    fn visit_item_union(&mut self, _: &syn::ItemUnion) {}
    fn visit_item_use(&mut self, _: &syn::ItemUse) {}
    fn visit_label(&mut self, _: &syn::Label) {}
    fn visit_lifetime(&mut self, _: &syn::Lifetime) {}
    fn visit_lifetime_def(&mut self, _: &syn::LifetimeDef) {}
    fn visit_lit(&mut self, _: &syn::Lit) {}
    fn visit_lit_bool(&mut self, _: &syn::LitBool) {}
    fn visit_lit_byte(&mut self, _: &syn::LitByte) {}
    fn visit_lit_byte_str(&mut self, _: &syn::LitByteStr) {}
    fn visit_lit_char(&mut self, _: &syn::LitChar) {}
    fn visit_lit_float(&mut self, _: &syn::LitFloat) {}
    fn visit_lit_int(&mut self, _: &syn::LitInt) {}
    fn visit_lit_str(&mut self, _: &syn::LitStr) {}
    fn visit_local(&mut self, _: &syn::Local) {}
    fn visit_macro(&mut self, _: &syn::Macro) {}
    fn visit_macro_delimiter(&mut self, _: &syn::MacroDelimiter) {}
    fn visit_member(&mut self, _: &syn::Member) {}
    fn visit_meta(&mut self, _: &syn::Meta) {}
    fn visit_meta_list(&mut self, _: &syn::MetaList) {}
    fn visit_meta_name_value(&mut self, _: &syn::MetaNameValue) {}
    fn visit_method_turbofish(&mut self, _: &syn::MethodTurbofish) {}
    fn visit_nested_meta(&mut self, _: &syn::NestedMeta) {}
    fn visit_parenthesized_generic_arguments(&mut self, _: &syn::ParenthesizedGenericArguments) {}
    fn visit_pat(&mut self, _: &syn::Pat) {}
    fn visit_pat_box(&mut self, _: &syn::PatBox) {}
    fn visit_pat_ident(&mut self, _: &syn::PatIdent) {}
    fn visit_pat_lit(&mut self, _: &syn::PatLit) {}
    fn visit_pat_macro(&mut self, _: &syn::PatMacro) {}
    fn visit_pat_or(&mut self, _: &syn::PatOr) {}
    fn visit_pat_path(&mut self, _: &syn::PatPath) {}
    fn visit_pat_range(&mut self, _: &syn::PatRange) {}
    fn visit_pat_reference(&mut self, _: &syn::PatReference) {}
    fn visit_pat_rest(&mut self, _: &syn::PatRest) {}
    fn visit_pat_slice(&mut self, _: &syn::PatSlice) {}
    fn visit_pat_struct(&mut self, _: &syn::PatStruct) {}
    fn visit_pat_tuple(&mut self, _: &syn::PatTuple) {}
    fn visit_pat_tuple_struct(&mut self, _: &syn::PatTupleStruct) {}
    fn visit_pat_type(&mut self, _: &syn::PatType) {}
    fn visit_pat_wild(&mut self, _: &syn::PatWild) {}
    fn visit_path(&mut self, _: &syn::Path) {}
    fn visit_path_arguments(&mut self, _: &syn::PathArguments) {}
    fn visit_path_segment(&mut self, _: &syn::PathSegment) {}
    fn visit_predicate_eq(&mut self, _: &syn::PredicateEq) {}
    fn visit_predicate_lifetime(&mut self, _: &syn::PredicateLifetime) {}
    fn visit_predicate_type(&mut self, _: &syn::PredicateType) {}
    fn visit_qself(&mut self, _: &syn::QSelf) {}
    fn visit_range_limits(&mut self, _: &syn::RangeLimits) {}
    fn visit_receiver(&mut self, _: &syn::Receiver) {}
    fn visit_return_type(&mut self, _: &syn::ReturnType) {}
    fn visit_signature(&mut self, _: &syn::Signature) {}
    fn visit_stmt(&mut self, _: &syn::Stmt) {}
    fn visit_trait_bound(&mut self, _: &syn::TraitBound) {}
    fn visit_trait_bound_modifier(&mut self, _: &syn::TraitBoundModifier) {}
    fn visit_trait_item(&mut self, _: &syn::TraitItem) {}
    fn visit_trait_item_const(&mut self, _: &syn::TraitItemConst) {}
    fn visit_trait_item_macro(&mut self, _: &syn::TraitItemMacro) {}
    fn visit_trait_item_method(&mut self, _: &syn::TraitItemMethod) {}
    fn visit_trait_item_type(&mut self, _: &syn::TraitItemType) {}
    fn visit_type(&mut self, _: &syn::Type) {}
    fn visit_type_array(&mut self, _: &syn::TypeArray) {}
    fn visit_type_bare_fn(&mut self, _: &syn::TypeBareFn) {}
    fn visit_type_group(&mut self, _: &syn::TypeGroup) {}
    fn visit_type_impl_trait(&mut self, _: &syn::TypeImplTrait) {}
    fn visit_type_infer(&mut self, _: &syn::TypeInfer) {}
    fn visit_type_macro(&mut self, _: &syn::TypeMacro) {}
    fn visit_type_never(&mut self, _: &syn::TypeNever) {}
    fn visit_type_param(&mut self, _: &syn::TypeParam) {}
    fn visit_type_param_bound(&mut self, _: &syn::TypeParamBound) {}
    fn visit_type_paren(&mut self, _: &syn::TypeParen) {}
    fn visit_type_path(&mut self, _: &syn::TypePath) {}
    fn visit_type_ptr(&mut self, _: &syn::TypePtr) {}
    fn visit_type_reference(&mut self, _: &syn::TypeReference) {}
    fn visit_type_slice(&mut self, _: &syn::TypeSlice) {}
    fn visit_type_trait_object(&mut self, _: &syn::TypeTraitObject) {}
    fn visit_type_tuple(&mut self, _: &syn::TypeTuple) {}
    fn visit_un_op(&mut self, _: &syn::UnOp) {}
    fn visit_use_glob(&mut self, _: &syn::UseGlob) {}
    fn visit_use_group(&mut self, _: &syn::UseGroup) {}
    fn visit_use_name(&mut self, _: &syn::UseName) {}
    fn visit_use_path(&mut self, _: &syn::UsePath) {}
    fn visit_use_rename(&mut self, _: &syn::UseRename) {}
    fn visit_use_tree(&mut self, _: &syn::UseTree) {}
    fn visit_variadic(&mut self, _: &syn::Variadic) {}
    fn visit_variant(&mut self, _: &syn::Variant) {}
    fn visit_vis_crate(&mut self, _: &syn::VisCrate) {}
    fn visit_vis_public(&mut self, _: &syn::VisPublic) {}
    fn visit_vis_restricted(&mut self, _: &syn::VisRestricted) {}
    fn visit_visibility(&mut self, _: &syn::Visibility) {}
    fn visit_where_clause(&mut self, _: &syn::WhereClause) {}
    fn visit_where_predicate(&mut self, _: &syn::WherePredicate) {}
}
