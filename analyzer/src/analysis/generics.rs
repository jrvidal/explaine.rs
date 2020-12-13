use super::NodeAnalyzer;
use crate::{
    help::{GenericsOf, HelpItem},
    ir::NodeId,
    syn_wrappers::{Syn, SynKind},
};
use quote::ToTokens;

const DISTANCE_TYPE_PARAM_TO_CONTAINER: usize = 3;

#[derive(Clone)]
pub(super) struct Generics {
    /// ids of the type parameters:
    /// - The ident of TypeParam
    pub types: Vec<NodeId>,
}

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn analyze_generics(&mut self, node: &syn::Generics) -> Option<Generics> {
        if node.lt_token.is_none() {
            return None;
        }

        let types = node
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(ty) => Some(ty),
                _ => None,
            })
            .filter_map(|ty| self.syn_to_id((&ty.ident).into()))
            .collect();

        Some(Generics { types })
    }
    pub(super) fn visit_const_param(&mut self, node: &syn::ConstParam) {
        if let Some(declaration) = self.find_containing_generics() {
            let (of, of_name) = (&declaration).into();
            return self.set_help(
                node,
                HelpItem::ConstParam {
                    name: node.ident.to_token_stream().to_string(),
                    of,
                    of_name,
                },
            );
        }
        #[cfg(features = "dev")]
        {
            panic!("Should find const param");
        }
        token![self, node.const_token, ConstParamSimple];
    }
    pub(super) fn visit_generic_param(&mut self, node: &syn::GenericParam) {
        let node = match node {
            syn::GenericParam::Lifetime(lifetime) => lifetime,
            _ => return,
        };

        let lifetime_range = (node.lifetime.apostrophe, node.lifetime.ident.span());

        if !self.between_spans(lifetime_range.0, lifetime_range.1) {
            return;
        }

        if let Some(declaration) =
            self.find_containing_generics_at(DISTANCE_TYPE_PARAM_TO_CONTAINER - 1)
        {
            let (of, of_name) = (&declaration).into();
            return self.set_help_between(
                lifetime_range.0,
                lifetime_range.1,
                HelpItem::LifetimeParam {
                    name: node.lifetime.ident.to_string(),
                    of,
                    of_name,
                },
            );
        }
    }

    pub(super) fn visit_predicate_type(&mut self, node: &syn::PredicateType) {
        token![self, node.lifetimes, BoundLifetimes];
    }
    pub(super) fn visit_type_param(&mut self, node: &syn::TypeParam) {
        if !self.between_spans(node.ident.span(), node.ident.span()) {
            return;
        }

        if let Some(declaration) = self.find_containing_generics() {
            let (of, of_name) = (&declaration).into();
            return self.set_help(
                &node.ident,
                HelpItem::TypeParam {
                    name: node.ident.to_token_stream().to_string(),
                    of,
                    of_name,
                },
            );
        }
    }
    pub(super) fn visit_where_clause(&mut self, node: &syn::WhereClause) {
        token![self, node.where_token, WhereClause];
    }

    fn find_containing_generics(&self) -> Option<Syn> {
        self.find_containing_generics_at(DISTANCE_TYPE_PARAM_TO_CONTAINER)
    }

    fn find_containing_generics_at(&self, height: usize) -> Option<Syn> {
        let (id, ancestor) = {
            let (id, ancestor) = self.get_ancestor(height)?;
            if ancestor.kind() == SynKind::Signature {
                self.get_ancestor(height + 1)?
            } else {
                (id, ancestor)
            }
        };

        self.generics_state.get_from_item(id).map(|_| ancestor)
    }
}

impl<'a, 'b> From<&'a Syn<'b>> for (GenericsOf, String) {
    fn from(node: &'a Syn<'b>) -> Self {
        match node {
            Syn::ItemStruct(item) => (GenericsOf::Struct, item.ident.to_string()),
            Syn::ItemEnum(item) => (GenericsOf::Enum, item.ident.to_string()),
            Syn::ItemUnion(item) => (GenericsOf::Union, item.ident.to_string()),
            Syn::ItemTrait(item) => (GenericsOf::Trait, item.ident.to_string()),
            Syn::ItemTraitAlias(item) => (GenericsOf::TraitAlias, item.ident.to_string()),
            Syn::ItemFn(item) => (GenericsOf::Fn, item.sig.ident.to_string()),
            Syn::ItemImpl(_) => (GenericsOf::Impl, "".to_string()),
            Syn::ImplItemMethod(item) => (GenericsOf::ImplMethod, item.sig.ident.to_string()),
            Syn::TraitItemMethod(item) => (GenericsOf::ImplMethod, item.sig.ident.to_string()),
            node => {
                debug_assert!(false, "unreachable {:?}", node.kind());
                (GenericsOf::Struct, "".to_string())
            }
        }
    }
}
