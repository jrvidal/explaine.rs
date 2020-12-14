use super::{generics::Generics, NodeAnalyzer};
use crate::{
    help::GenericsOf,
    syn_wrappers::{Syn, SynKind},
    HelpItem,
};
use quote::ToTokens;
use syn::spanned::Spanned;

impl<'a> NodeAnalyzer<'a> {
    pub(super) fn visit_type_array(&mut self, node: &syn::TypeArray) {
        if self.between_locations(node.span().start(), node.elem.span().start())
            || self.between_spans(node.semi_token.span(), node.span())
        {
            return self.set_help(node, HelpItem::TypeArray);
        }
    }
    pub(super) fn visit_type_bare_fn_first_pass(&mut self, node: &syn::TypeBareFn) {
        let lifetimes = node
            .lifetimes
            .as_ref()
            .map(|lf| lf.into())
            .and_then(|syn| self.syn_to_id(syn).map(|id| (id, syn)));
        if let Some((id, syn)) = lifetimes {
            self.fill_generics_info(id, syn, false);
        }
    }
    pub(super) fn visit_type_bare_fn(&mut self, node: &syn::TypeBareFn) {
        token![self, some node.abi, TypeBareFnAbi];
        token![self, some node.unsafety, TypeBareUnsafeFn];
        token![self, some node.lifetimes, BoundLifetimesBareFnType];
        return self.set_help(node, HelpItem::TypeBareFn);
    }
    pub(super) fn visit_type_impl_trait(&mut self, node: &syn::TypeImplTrait) {
        token![self, node.impl_token, TypeImplTrait];
    }
    pub(super) fn visit_type_infer(&mut self, node: &syn::TypeInfer) {
        return self.set_help(node, HelpItem::TypeInfer);
    }
    pub(super) fn visit_type_never(&mut self, node: &syn::TypeNever) {
        return self.set_help(node, HelpItem::TypeNever);
    }
    pub(super) fn visit_type_path(&mut self, node: &syn::TypePath) {
        if let Some(item) = well_known_type(node) {
            // SHORTCUT
            if let HelpItem::KnownTypeStr = item {
                if self.has_ancestor(2, SynKind::TypeReference) {
                    return;
                }
            }
            return self.set_help(node, item);
        }

        // TODO: false positives, item declarations might shadow the parameter
        if let Some(ident) = node.path.get_ident() {
            let find_generics = |generics: &Generics| {
                // self.generics_state.get_from_item(id).filter(|generics| {
                generics
                    .types
                    .iter()
                    .filter_map(|&ident_id| self.id_to_syn(ident_id))
                    .any(|syn| match syn {
                        Syn::TypeParam(param) => &param.ident == ident,
                        _ => false,
                    })
                // })
            };

            let help = if let Some(declaration) = self
                .generics_state
                .declarations()
                .filter(|(_, gen)| find_generics(gen))
                .next()
                .and_then(|(item_id, _)| self.id_to_syn(item_id))
            {
                let (of, of_name) = (&declaration).into();
                HelpItem::TypeParamUse {
                    of,
                    of_name,
                    name: ident.to_string(),
                    implementation: match of {
                        GenericsOf::Impl => true,
                        _ => false,
                    },
                }
            } else {
                return;
            };

            self.set_help(node, help);
        }
    }
    pub(super) fn visit_type_ptr(&mut self, node: &syn::TypePtr) {
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
    pub(super) fn visit_type_reference(&mut self, node: &syn::TypeReference) {
        if let Some(lifetime) = &node.lifetime {
            if self.within(lifetime) {}
        }

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

        // TODO: see [HITBOX]. Used to have only the span of `&mut 'a` as clickable
        return self.set_help(
            &node,
            HelpItem::TypeReference {
                lifetime: node.lifetime.is_some(),
                mutable: node.mutability.is_some(),
                ty: node.elem.to_token_stream().to_string(),
            },
        );
    }
    pub(super) fn visit_type_slice(&mut self, node: &syn::TypeSlice) {
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
    pub(super) fn visit_type_trait_object(&mut self, node: &syn::TypeTraitObject) {
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
                lifetime: lifetime.map(|lt| lt.to_string()),
                multiple,
                dyn_: node.dyn_token.is_some(),
                ty: ty.path.to_token_stream().to_string(),
            },
        );
    }
    pub(super) fn visit_type_tuple(&mut self, node: &syn::TypeTuple) {
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
}

fn well_known_type(type_path: &syn::TypePath) -> Option<HelpItem> {
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
