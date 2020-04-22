use crate::help::*;
use crate::ir::{Location, Ptr, PtrData};
use proc_macro2::{LineColumn, Span};
use quote::ToTokens;
use std::collections::HashMap;
use syn::spanned::Spanned;

pub struct Analyzer {
    pub(crate) id_to_ptr: HashMap<usize, PtrData>,
    pub(crate) ptr_to_id: HashMap<Ptr, usize>,
    pub(crate) locations: Vec<(usize, Location)>,
}

macro_rules! analyze {
    ($self:ident, $node:ident, $ty:path, $method:ident) => {
        if let Some(node) = $node.downcast::<$ty>() {
            // println!("downcasted to {:?}", stringify!($ty));
            $self.$method(node);
            return;
        }
    };
}

struct NodeAnalyzer<'a> {
    id: usize,
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
        // println!("{:?}, {:#?}", location, self.locations);
        let result = self
            .locations
            .binary_search_by(|(_, loc)| loc.cmp(&location))
            .map(|idx| Some(idx))
            .unwrap_or_else(|err| if err == 0 { None } else { Some(err - 1) });

        let id = if let Some((id, _)) = result.and_then(|idx| self.locations.get(idx)) {
            *id
        } else {
            return None;
        };

        // println!("found id #{}", id);

        let mut node = if let Some(node) = self.id_to_ptr.get(&id) {
            node
        } else {
            return None;
        };

        let mut node_analyzer = NodeAnalyzer {
            location,
            id_to_ptr: &self.id_to_ptr,
            help: None,
            id,
        };

        loop {
            node_analyzer.analyze_node(&node.ptr);

            if let Some(((start, end), help)) = node_analyzer.help {
                break Some(AnalysisResult { start, end, help });
            }

            if let Some(parent) = self.id_to_ptr.get(&node.parent) {
                node_analyzer.id = node.parent;
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
        // OMITTED: handled upstream
        // analyze![self, node, syn::Abi, visit_abi];
        analyze![
            self,
            node,
            syn::AngleBracketedGenericArguments,
            visit_angle_bracketed_generic_arguments
        ];
        analyze![self, node, syn::Arm, visit_arm];
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

    fn attributes(&self, id: usize) -> Option<&[syn::Attribute]> {
        let data = if let Some(data) = self.id_to_ptr.get(&id) {
            data
        } else {
            return None;
        };

        macro_rules! get_attr {
            ($ptr:ident, $ty:path) => {
                if let Some(node) = $ptr.ptr.downcast::<$ty>() {
                    return Some(&node.attrs[..]);
                }
            };
        }

        get_attr![data, syn::Arm];
        get_attr![data, syn::BareFnArg];
        get_attr![data, syn::ConstParam];
        get_attr![data, syn::ExprArray];
        get_attr![data, syn::ExprAssign];
        get_attr![data, syn::ExprAssignOp];
        get_attr![data, syn::ExprAsync];
        get_attr![data, syn::ExprAwait];
        get_attr![data, syn::ExprBinary];
        get_attr![data, syn::ExprBlock];
        get_attr![data, syn::ExprBox];
        get_attr![data, syn::ExprBreak];
        get_attr![data, syn::ExprCall];
        get_attr![data, syn::ExprCast];
        get_attr![data, syn::ExprClosure];
        get_attr![data, syn::ExprContinue];
        get_attr![data, syn::ExprField];
        get_attr![data, syn::ExprForLoop];
        get_attr![data, syn::ExprGroup];
        get_attr![data, syn::ExprIf];
        get_attr![data, syn::ExprIndex];
        get_attr![data, syn::ExprLet];
        get_attr![data, syn::ExprLit];
        get_attr![data, syn::ExprLoop];
        get_attr![data, syn::ExprMacro];
        get_attr![data, syn::ExprMatch];
        get_attr![data, syn::ExprMethodCall];
        get_attr![data, syn::ExprParen];
        get_attr![data, syn::ExprPath];
        get_attr![data, syn::ExprRange];
        get_attr![data, syn::ExprReference];
        get_attr![data, syn::ExprRepeat];
        get_attr![data, syn::ExprReturn];
        get_attr![data, syn::ExprStruct];
        get_attr![data, syn::ExprTry];
        get_attr![data, syn::ExprTryBlock];
        get_attr![data, syn::ExprTuple];
        get_attr![data, syn::ExprType];
        get_attr![data, syn::ExprUnary];
        get_attr![data, syn::ExprUnsafe];
        get_attr![data, syn::ExprWhile];
        get_attr![data, syn::ExprYield];
        get_attr![data, syn::Field];
        get_attr![data, syn::FieldPat];
        get_attr![data, syn::FieldValue];
        get_attr![data, syn::File];
        get_attr![data, syn::ForeignItemFn];
        get_attr![data, syn::ForeignItemMacro];
        get_attr![data, syn::ForeignItemStatic];
        get_attr![data, syn::ForeignItemType];
        get_attr![data, syn::ImplItemConst];
        get_attr![data, syn::ImplItemMacro];
        get_attr![data, syn::ImplItemMethod];
        get_attr![data, syn::ImplItemType];
        get_attr![data, syn::ItemConst];
        get_attr![data, syn::ItemEnum];
        get_attr![data, syn::ItemExternCrate];
        get_attr![data, syn::ItemFn];
        get_attr![data, syn::ItemForeignMod];
        get_attr![data, syn::ItemImpl];
        get_attr![data, syn::ItemMacro];
        get_attr![data, syn::ItemMacro2];
        get_attr![data, syn::ItemMod];
        get_attr![data, syn::ItemStatic];
        get_attr![data, syn::ItemStruct];
        get_attr![data, syn::ItemTrait];
        get_attr![data, syn::ItemTraitAlias];
        get_attr![data, syn::ItemType];
        get_attr![data, syn::ItemUnion];
        get_attr![data, syn::ItemUse];
        get_attr![data, syn::LifetimeDef];
        get_attr![data, syn::Local];
        get_attr![data, syn::PatBox];
        get_attr![data, syn::PatIdent];
        get_attr![data, syn::PatLit];
        get_attr![data, syn::PatMacro];
        get_attr![data, syn::PatOr];
        get_attr![data, syn::PatPath];
        get_attr![data, syn::PatRange];
        get_attr![data, syn::PatReference];
        get_attr![data, syn::PatSlice];
        get_attr![data, syn::PatStruct];
        get_attr![data, syn::PatTuple];
        get_attr![data, syn::PatTupleStruct];
        get_attr![data, syn::PatType];
        get_attr![data, syn::PatWild];
        get_attr![data, syn::Receiver];
        get_attr![data, syn::TraitItemConst];
        get_attr![data, syn::TraitItemMacro];
        get_attr![data, syn::TraitItemMethod];
        get_attr![data, syn::TraitItemType];
        get_attr![data, syn::TypeParam];
        get_attr![data, syn::Variadic];
        get_attr![data, syn::Variant];

        None
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
        super::within_locations(
            LineColumn {
                line: loc.line,
                column: loc.column,
            },
            start,
            end,
        )
    }

    fn has_ancestor<T: 'static>(&self, ancestor: usize) -> bool {
        self.get_ancestor::<T>(ancestor).is_some()
    }

    fn get_ancestor<T: 'static>(&self, ancestor: usize) -> Option<&T> {
        let mut id = self.id;

        // println!("get_ancestor(#{})", id);

        for _ in 0..ancestor {
            id = if let Some(parent_id) = self.id_to_ptr.get(&id).map(|data| data.parent) {
                // dbg!(parent_id);
                parent_id
            } else {
                // println!("bailing out");
                return None;
            }
        }

        // println!("trying with #{}", id);
        self.id_to_ptr
            .get(&id)
            .and_then(|data| data.ptr.downcast::<T>())
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
        let outer = outer_attr(node);

        if !node.path.is_ident("doc") {
            return self.set_help(&node, HelpItem::Attribute { outer });
        }

        let attributes = self
            .id_to_ptr
            .get(&self.id)
            .map(|data| data.parent)
            .and_then(|parent_id| self.attributes(parent_id))
            .unwrap_or(&[]);

        let bounds = if attributes.len() > 0 {
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
    fn visit_bare_fn_arg(&mut self, _: &syn::BareFnArg) {}
    fn visit_bin_op(&mut self, _: &syn::BinOp) {}
    fn visit_binding(&mut self, _: &syn::Binding) {}
    fn visit_block(&mut self, _: &syn::Block) {}
    fn visit_bound_lifetimes(&mut self, _: &syn::BoundLifetimes) {}
    fn visit_const_param(&mut self, _: &syn::ConstParam) {}
    fn visit_constraint(&mut self, _: &syn::Constraint) {}
    fn visit_expr(&mut self, _: &syn::Expr) {}
    fn visit_expr_array(&mut self, node: &syn::ExprArray) {
        if !self.has_ancestor::<syn::ExprReference>(2) {
            self.set_help(node, HelpItem::ExprArray)
        }
    }
    fn visit_expr_assign(&mut self, _: &syn::ExprAssign) {}
    fn visit_expr_assign_op(&mut self, _: &syn::ExprAssignOp) {}
    fn visit_expr_async(&mut self, _: &syn::ExprAsync) {}
    fn visit_expr_await(&mut self, _: &syn::ExprAwait) {}
    fn visit_expr_binary(&mut self, _: &syn::ExprBinary) {}
    fn visit_expr_block(&mut self, _: &syn::ExprBlock) {}
    fn visit_expr_box(&mut self, _: &syn::ExprBox) {}
    fn visit_expr_break(&mut self, node: &syn::ExprBreak) {
        return self.set_help(
            &node,
            HelpItem::ExprBreak {
                expr: node.expr.is_some(),
                label: node.label.as_ref().map(|l| l.to_string()),
            },
        );
    }
    fn visit_expr_call(&mut self, _: &syn::ExprCall) {}
    fn visit_expr_cast(&mut self, _: &syn::ExprCast) {}
    fn visit_expr_closure(&mut self, _: &syn::ExprClosure) {}
    fn visit_expr_continue(&mut self, _: &syn::ExprContinue) {}
    fn visit_expr_field(&mut self, _: &syn::ExprField) {}
    fn visit_expr_for_loop(&mut self, node: &syn::ExprForLoop) {
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
    fn visit_expr_group(&mut self, _: &syn::ExprGroup) {}
    fn visit_expr_if(&mut self, _: &syn::ExprIf) {}
    fn visit_expr_index(&mut self, _: &syn::ExprIndex) {}
    fn visit_expr_let(&mut self, _: &syn::ExprLet) {}
    fn visit_expr_lit(&mut self, _: &syn::ExprLit) {}
    fn visit_expr_loop(&mut self, node: &syn::ExprLoop) {
        token![self, node.loop_token, ExprLoopToken];
    }
    fn visit_expr_macro(&mut self, _: &syn::ExprMacro) {}
    fn visit_expr_match(&mut self, _: &syn::ExprMatch) {}
    fn visit_expr_method_call(&mut self, _: &syn::ExprMethodCall) {}
    fn visit_expr_paren(&mut self, _: &syn::ExprParen) {}
    fn visit_expr_path(&mut self, _: &syn::ExprPath) {}
    fn visit_expr_range(&mut self, _: &syn::ExprRange) {}
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
    fn visit_expr_repeat(&mut self, _: &syn::ExprRepeat) {}
    fn visit_expr_return(&mut self, _: &syn::ExprReturn) {}
    fn visit_expr_struct(&mut self, _: &syn::ExprStruct) {}
    fn visit_expr_try(&mut self, _: &syn::ExprTry) {}
    fn visit_expr_try_block(&mut self, _: &syn::ExprTryBlock) {}
    fn visit_expr_tuple(&mut self, _: &syn::ExprTuple) {}
    fn visit_expr_type(&mut self, _: &syn::ExprType) {}
    fn visit_expr_unary(&mut self, _: &syn::ExprUnary) {}
    fn visit_expr_unsafe(&mut self, _: &syn::ExprUnsafe) {}
    fn visit_expr_while(&mut self, node: &syn::ExprWhile) {
        let while_let = if let syn::Expr::Let(..) = *node.cond {
            true
        } else {
            false
        };

        token![self, some node.label, * HelpItem::Label {
            loop_of: if while_let { LoopOf::WhileLet } else { LoopOf::While },
        }];

        token![
            self,
            node.while_token,
            *if while_let {
                HelpItem::ExprWhileLet
            } else {
                HelpItem::ExprWhile
            }
        ];
    }
    fn visit_expr_yield(&mut self, _: &syn::ExprYield) {}
    fn visit_field(&mut self, node: &syn::Field) {
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
    fn visit_field_pat(&mut self, _: &syn::FieldPat) {}
    fn visit_field_value(&mut self, _: &syn::FieldValue) {}
    fn visit_fields(&mut self, _: &syn::Fields) {}
    fn visit_fields_named(&mut self, _: &syn::FieldsNamed) {}
    fn visit_fields_unnamed(&mut self, _: &syn::FieldsUnnamed) {}
    fn visit_file(&mut self, _: &syn::File) {}
    fn visit_fn_arg(&mut self, node: &syn::FnArg) {
        if let Some(sig) = self.get_ancestor::<syn::Signature>(1) {
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
    fn visit_foreign_item(&mut self, _: &syn::ForeignItem) {}
    fn visit_foreign_item_fn(&mut self, _: &syn::ForeignItemFn) {}
    fn visit_foreign_item_macro(&mut self, _: &syn::ForeignItemMacro) {}
    fn visit_foreign_item_static(&mut self, _: &syn::ForeignItemStatic) {}
    fn visit_foreign_item_type(&mut self, _: &syn::ForeignItemType) {}
    fn visit_generic_argument(&mut self, _: &syn::GenericArgument) {}
    fn visit_generic_method_argument(&mut self, _: &syn::GenericMethodArgument) {}
    fn visit_generic_param(&mut self, _: &syn::GenericParam) {}
    fn visit_generics(&mut self, _: &syn::Generics) {}
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
    fn visit_impl_item(&mut self, _: &syn::ImplItem) {}
    fn visit_impl_item_const(&mut self, _: &syn::ImplItemConst) {}
    fn visit_impl_item_macro(&mut self, _: &syn::ImplItemMacro) {}
    fn visit_impl_item_method(&mut self, node: &syn::ImplItemMethod) {
        if !self.between(&node.sig.fn_token, &node.sig.ident) {
            return;
        }

        let is_method = receiver_help(&node.sig).is_some();
        let trait_ = self
            .get_ancestor::<syn::ItemImpl>(2)
            .and_then(|item| item.trait_.as_ref());

        let of = if is_method {
            FnOf::Method
        } else {
            FnOf::AssociatedFunction
        };

        if let Some(impl_) = self.get_ancestor::<syn::ItemImpl>(2) {
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
            // token![
            //     self,
            //     node.sig.fn_token => node.sig.ident,
            //     * HelpItem::ImplItemMethod {
            //         of,
            //     }
            // ];
        }
    }
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
    fn visit_item_use(&mut self, node: &syn::ItemUse) {
        token![self, some node.leading_colon, PathLeadingColon];
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemUse);
    }
    fn visit_label(&mut self, node: &syn::Label) {
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

        return self.set_help(&node, HelpItem::Label { loop_of });
    }
    fn visit_lifetime(&mut self, _: &syn::Lifetime) {}
    fn visit_lifetime_def(&mut self, _: &syn::LifetimeDef) {}
    fn visit_lit(&mut self, _: &syn::Lit) {}
    fn visit_lit_bool(&mut self, _: &syn::LitBool) {}
    fn visit_lit_byte(&mut self, _: &syn::LitByte) {}
    fn visit_lit_byte_str(&mut self, _: &syn::LitByteStr) {}
    fn visit_lit_char(&mut self, _: &syn::LitChar) {}
    fn visit_lit_float(&mut self, _: &syn::LitFloat) {}
    fn visit_lit_int(&mut self, node: &syn::LitInt) {
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
    fn visit_lit_str(&mut self, _: &syn::LitStr) {}
    fn visit_local(&mut self, node: &syn::Local) {
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
    fn visit_pat_tuple(&mut self, node: &syn::PatTuple) {
        if self.get_ancestor::<syn::PatTupleStruct>(1).is_some() {
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
    fn visit_pat_type(&mut self, _: &syn::PatType) {}
    fn visit_pat_wild(&mut self, _: &syn::PatWild) {}
    fn visit_path(&mut self, node: &syn::Path) {
        let qself = loop {
            let ancestor = if let Some(ancestor) = self
                .id_to_ptr
                .get(&self.id)
                .and_then(|data| self.id_to_ptr.get(&data.parent))
            {
                ancestor
            } else {
                return;
            };

            if let Some(expr_path) = ancestor.ptr.downcast::<syn::ExprPath>() {
                break &expr_path.qself;
            }
            if let Some(type_path) = ancestor.ptr.downcast::<syn::TypePath>() {
                break &type_path.qself;
            }
            if let Some(pat_path) = ancestor.ptr.downcast::<syn::PatPath>() {
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
    }
    fn visit_path_arguments(&mut self, _: &syn::PathArguments) {}
    fn visit_path_segment(&mut self, _: &syn::PathSegment) {}
    fn visit_predicate_eq(&mut self, _: &syn::PredicateEq) {}
    fn visit_predicate_lifetime(&mut self, _: &syn::PredicateLifetime) {}
    fn visit_predicate_type(&mut self, _: &syn::PredicateType) {}
    fn visit_qself(&mut self, _: &syn::QSelf) {}
    fn visit_range_limits(&mut self, _: &syn::RangeLimits) {}
    fn visit_receiver(&mut self, _: &syn::Receiver) {}
    fn visit_return_type(&mut self, node: &syn::ReturnType) {
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
    }
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
    fn visit_type_bare_fn(&mut self, node: &syn::TypeBareFn) {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        token![self, some node.lifetimes, BoundLifetimesBareFnType];
        return self.set_help(node, HelpItem::TypeBareFn);
    }
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
    fn visit_type_tuple(&mut self, node: &syn::TypeTuple) {
        if node.elems.is_empty() {
            return self.set_help(node, HelpItem::TypeTupleUnit);
        }
        return self.set_help(node, HelpItem::TypeTuple {
            single_comma: node.elems.len() == 1 && node.elems.trailing_punct()
        });
    }
    fn visit_un_op(&mut self, _: &syn::UnOp) {}
    fn visit_use_glob(&mut self, _: &syn::UseGlob) {}
    fn visit_use_group(&mut self, _: &syn::UseGroup) {}
    fn visit_use_name(&mut self, node: &syn::UseName) {
        if node.ident != "self" {
            return;
        }

        if self.get_ancestor::<syn::UseGroup>(2).is_none() {
            return;
        }

        let parent = if let Some(path) = self.get_ancestor::<syn::UsePath>(4) {
            path.ident.to_string()
        } else {
            return;
        };

        return self.set_help(node, HelpItem::UseGroupSelf { parent });
    }
    fn visit_use_path(&mut self, node: &syn::UsePath) {
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
    }
    fn visit_use_rename(&mut self, _: &syn::UseRename) {}
    fn visit_use_tree(&mut self, _: &syn::UseTree) {}
    fn visit_variadic(&mut self, _: &syn::Variadic) {}
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
        let name = if let Some(item_enum) = self.get_ancestor::<syn::ItemEnum>(1) {
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
    fn visit_vis_crate(&mut self, _: &syn::VisCrate) {}
    fn visit_vis_public(&mut self, _: &syn::VisPublic) {}
    fn visit_vis_restricted(&mut self, _: &syn::VisRestricted) {}
    fn visit_visibility(&mut self, _: &syn::Visibility) {}
    fn visit_where_clause(&mut self, _: &syn::WhereClause) {}
    fn visit_where_predicate(&mut self, _: &syn::WherePredicate) {}
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
                        let mut id = analyzer.id;
                        let method = loop {
                            id = if let Some(parent) =
                                analyzer.id_to_ptr.get(&id).map(|data| data.parent)
                            {
                                parent
                            } else {
                                break None;
                            };

                            let ptr = if let Some(data) = analyzer.id_to_ptr.get(&id) {
                                &data.ptr
                            } else {
                                break None;
                            };

                            if let Some(method) = ptr.downcast::<syn::ImplItemMethod>() {
                                break Some(method.sig.ident.to_string());
                            }
                            if let Some(method) = ptr.downcast::<syn::TraitItemMethod>() {
                                break Some(method.sig.ident.to_string());
                            }
                        };

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
    if analyzer.has_ancestor::<syn::Local>(2)
        || (analyzer.has_ancestor::<syn::Local>(4) && analyzer.has_ancestor::<syn::PatType>(2))
    {
        return Some(BindingOf::Let);
    }
    if analyzer.has_ancestor::<syn::FnArg>(4) {
        return Some(BindingOf::Arg);
    }

    None
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
