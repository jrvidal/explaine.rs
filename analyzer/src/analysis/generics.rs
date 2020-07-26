use super::{NodeAnalyzer, Ptr};
use crate::{
    help::{self, HelpItem},
    syn_wrappers::Syn,
};
use quote::ToTokens;

const DISTANCE_TYPE_PARAM_TO_CONTAINER: usize = 3;

#[derive(Default, Clone)]
pub(super) struct Generics {
    pub type_: usize,
    pub lifetime: usize,
    pub const_: usize,
}

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn analyze_generics(&mut self, node: &syn::Generics) -> Option<Generics> {
        if node.lt_token.is_none() {
            return None;
        }
        let mut generics = Generics::default();
        for param in &node.params {
            let count = match param {
                syn::GenericParam::Type(_) => &mut generics.type_,
                syn::GenericParam::Lifetime(_) => &mut generics.lifetime,
                syn::GenericParam::Const(_) => &mut generics.const_,
            };
            *count = *count + 1;
        }
        Some(generics)
    }
    pub(super) fn visit_const_param(&mut self, node: &syn::ConstParam) {
        token![self, node.const_token, ConstParam];
    }
    pub(super) fn visit_type_param_first_pass(&mut self, _: &syn::TypeParam) {
        let scope_id = match self.get_ancestor(DISTANCE_TYPE_PARAM_TO_CONTAINER) {
            Some(i @ Syn::ItemStruct(_)) => self.ptr_to_id.get(&Ptr::new(i.clone())).cloned(),
            _ => None,
        };
        if let Some(scope_id) = scope_id {
            self.state.insert(self.id, scope_id);
        }
    }
    pub(super) fn visit_predicate_type(&mut self, node: &syn::PredicateType) {
        token![self, node.lifetimes, BoundLifetimes];
    }
    pub(super) fn visit_type_param(&mut self, node: &syn::TypeParam) {
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
    pub(super) fn visit_where_clause(&mut self, node: &syn::WhereClause) {
        token![self, node.where_token, WhereClause];
    }
}

impl<'a> From<&'a Generics> for help::Generics {
    fn from(generics: &'a Generics) -> Self {
        Self {
            type_: generics.type_ > 0,
            lifetime: generics.lifetime > 0,
            const_: generics.const_ > 0,
        }
    }
}
