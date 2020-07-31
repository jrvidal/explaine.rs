use super::NodeAnalyzer;
use crate::help::{GenericsOf, HelpItem};
use proc_macro2::Span;
use syn::spanned::Spanned;

macro_rules! generics {
    ($self:expr, $node:expr) => {
        $self.fill_generics_info($self.id, &$node.generics, true);
    };
}

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn visit_item_const(&mut self, node: &syn::ItemConst) {
        token![self, node.const_token, ItemConst];
    }
    pub(super) fn visit_item_enum_first_pass(&mut self, node: &syn::ItemEnum) {
        generics![self, node];
    }
    pub(super) fn visit_item_enum(&mut self, node: &syn::ItemEnum) {
        token![self, node.enum_token => node.ident, * HelpItem::ItemEnum {
            empty: node.variants.is_empty(),
            generic: self.generics_for(self.id).is_some()
        }];
    }
    pub(super) fn visit_item_extern_crate(&mut self, node: &syn::ItemExternCrate) {
        if let Some((as_token, _)) = node.rename {
            token![self, as_token, AsRenameExternCrate];
        }
        token![self, node.extern_token, ItemExternCrate];
    }
    pub(super) fn visit_item_fn_first_pass(&mut self, node: &syn::ItemFn) {
        self.fill_generics_info(self.id, &node.sig.generics, true);
    }
    pub(super) fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        token![self, node.sig.fn_token => node.sig.ident, ItemFn];
    }
    pub(super) fn visit_item_foreign_mod(&mut self, node: &syn::ItemForeignMod) {
        token![self, node.abi, ItemForeignModAbi];
    }
    pub(super) fn visit_item_impl_first_pass(&mut self, node: &syn::ItemImpl) {
        generics![self, node];
    }
    pub(super) fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
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
    pub(super) fn visit_item_macro(&mut self, node: &syn::ItemMacro) {
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
    pub(super) fn visit_item_mod(&mut self, node: &syn::ItemMod) {
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
    pub(super) fn visit_item_static(&mut self, node: &syn::ItemStatic) {
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
    pub(super) fn visit_item_struct_first_pass(&mut self, node: &syn::ItemStruct) {
        generics![self, node];
    }
    pub(super) fn visit_item_struct(&mut self, node: &syn::ItemStruct) {
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
                    generic: self.generics_for(self.id).is_some(),
                },
            );
        }
    }
    pub(super) fn visit_item_trait_first_pass(&mut self, node: &syn::ItemTrait) {
        generics![self, node];
    }
    pub(super) fn visit_item_trait(&mut self, node: &syn::ItemTrait) {
        token![self, some node.unsafety, ItemUnsafeTrait];
        token![self, some node.auto_token, ItemAutoTrait];
        token![self, node.trait_token => node.ident, * HelpItem::ItemTrait {
            generic: self.generics_for(self.id).is_some()
        } ];
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
    pub(super) fn visit_item_trait_alias_first_pass(&mut self, node: &syn::ItemTraitAlias) {
        generics![self, node];
    }
    pub(super) fn visit_item_trait_alias(&mut self, node: &syn::ItemTraitAlias) {
        token![self, node.trait_token, ItemTraitAlias];
    }
    pub(super) fn visit_item_type(&mut self, node: &syn::ItemType) {
        token![self, node.type_token => node.ident, ItemType];
    }
    pub(super) fn visit_item_union_first_pass(&mut self, node: &syn::ItemUnion) {
        generics![self, node];
    }
    pub(super) fn visit_item_union(&mut self, node: &syn::ItemUnion) {
        token![
            self,
            node.union_token,
            *HelpItem::ItemUnion {
                generic: self.generics_for(self.id).is_some()
            }
        ];
    }
    pub(super) fn visit_item_use(&mut self, node: &syn::ItemUse) {
        token![self, some node.leading_colon, PathLeadingColon];
        let start = vis_span(&node.vis).unwrap_or_else(|| node.use_token.span());
        return self.set_help_between(start, node.span(), HelpItem::ItemUse);
    }
}

fn vis_span(vis: &syn::Visibility) -> Option<Span> {
    if let syn::Visibility::Inherited = vis {
        None
    } else {
        Some(vis.span())
    }
}
