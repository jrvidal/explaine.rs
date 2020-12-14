use super::{receiver_help, NodeAnalyzer};
use crate::help::{FnOf, HelpItem};
use crate::syn_wrappers::Syn;
use quote::ToTokens;
use syn::spanned::Spanned;

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn visit_impl_item_const(&mut self, node: &syn::ImplItemConst) {
        token![self, node.const_token, ImplItemConst];
    }
    pub(super) fn visit_impl_item_method_first_pass(&mut self, node: &syn::ImplItemMethod) {
        self.fill_generics_info(self.id, (&node.sig.generics).into(), false);
    }
    pub(super) fn visit_impl_item_method(&mut self, node: &syn::ImplItemMethod) {
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
    pub(super) fn visit_impl_item_type(&mut self, node: &syn::ImplItemType) {
        token![self, node.type_token, ImplItemType];
    }
    pub(super) fn visit_trait_item_const(&mut self, node: &syn::TraitItemConst) {
        token![self, node.const_token, TraitItemConst];
    }
    pub(super) fn visit_trait_item_method_first_pass(&mut self, node: &syn::TraitItemMethod) {
        self.fill_generics_info(self.id, (&node.sig.generics).into(), false);
    }
    pub(super) fn visit_trait_item_method(&mut self, node: &syn::TraitItemMethod) {
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
    pub(super) fn visit_trait_item_type(&mut self, node: &syn::TraitItemType) {
        token![self, node.type_token, TraitItemType];
    }
}
