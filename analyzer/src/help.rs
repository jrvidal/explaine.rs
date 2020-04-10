#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "dev")]
use std::fmt::Debug;

std::thread_local! {
    static TEMPLATE: tinytemplate::TinyTemplate<'static> = init_template();
}

struct HelpData {
    template: &'static str,
    title: &'static str,
    book: Option<&'static str>,
    keyword: Option<&'static str>,
    std: Option<&'static str>,
}

std::include!(concat!(env!("OUT_DIR"), "/help.rs"));
#[cfg_attr(all(not(test), not(feature = "dev")), derive(Serialize))]
#[cfg_attr(test, derive(Debug, Clone, Serialize, Deserialize, PartialEq))]
#[cfg_attr(feature = "dev", derive(Debug, Clone, Serialize))]
#[serde(tag = "type")]
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
        of: RestOf,
    },
    PatStruct {
        empty: bool,
        bindings: Option<BindingOf>,
    },
    PatTuple {
        bindings: Option<BindingOf>,
    },
    PatTupleStruct {
        bindings: Option<BindingOf>,
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
        of: FnOf,
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
        fields: Option<Fields>,
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
    True,
    False,
    LitByteStr,
    LitFloat {
        suffix: Option<String>,
        separators: bool,
    },
    LitInt {
        suffix: Option<String>,
        mode: Option<IntMode>,
        prefix: Option<String>,
        separators: bool,
    },
    LitStr,
    ExprForLoopToken,
    BoundLifetimes,
    BoundLifetimesTraitBound {
        lifetime: String,
        ty: String,
        multiple: bool,
    },
    BoundLifetimesBareFnType,
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
    RawIdent,
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
    TypeBareFn,
    TypeBareFnAbi,
    TypeBareUnsafeFn,
    TypeInfer,
    TypeNever,
    TypeParamBoundAdd,
    TypeTupleUnit,
    TypeTuple,
    TypeConstPtr,
    TypeMutPtr,
    WhereClause,
    ItemTraitAlias,
    Field {
        name: Option<String>,
        of: FieldOf,
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
    RArrow {
        return_of: ReturnOf,
    },
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

macro_rules! variant {
    ($item:item) => {
        #[cfg_attr(all(not(test), not(feature = "dev")), derive(Serialize, Copy, Clone))]
        #[cfg_attr(test, derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq))]
        #[cfg_attr(feature = "dev", derive(Serialize, Copy, Clone, Debug))]
        #[serde(rename_all(serialize = "lowercase"))]
        $item
    };
}

variant![
    pub enum FieldOf {
        #[serde(rename(serialize = "enum variant"))]
        Variant,
        Union,
        Struct,
    }
];

variant![
    pub enum BindingOf {
        Let,
        Arg,
    }
];

variant![
    pub enum IntMode {
        Binary,
        Hexadecimal,
        Octal,
    }
];

variant![
    pub enum RestOf {
        Struct,
        Tuple,
        #[serde(rename(serialize = "tuple struct"))]
        TupleStruct,
    }
];

variant![
    pub enum FnOf {
        Method,
        Function,
        #[serde(rename(serialize = "trait method"))]
        TraitMethod,
    }
];

variant![
    pub enum Fields {
        Named,
        Unnamed,
    }
];

variant![
    pub enum ReturnOf {
        Function,
        Method,
        #[serde(rename(serialize = "bare function type"))]
        BareFunctionType,
        FnTrait,
        Closure,
    }
];

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
