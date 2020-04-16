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
    AddBinOp,
    SubBinOp,
    MulBinOp,
    DivBinOp,
    RemBinOp,
    AndBinOp,
    OrBinOp,
    BitXorBinOp,
    BitAndBinOp,
    BitOrBinOp,
    ShlBinOp,
    ShrBinOp,
    EqBinOp,
    LtBinOp,
    LeBinOp,
    NeBinOp,
    GeBinOp,
    GtBinOp,
    AddEqBinOp,
    SubEqBinOp,
    MulEqBinOp,
    DivEqBinOp,
    RemEqBinOp,
    BitXorEqBinOp,
    BitAndEqBinOp,
    BitOrEqBinOp,
    ShlEqBinOp,
    ShrEqBinOp,
    ItemUse,
    Macro,
    MacroTokens,
    Turbofish,
    ParenthesizedGenericArguments,
    PatBox,
    PatIdent {
        mutability: bool,
        by_ref: bool,
        ident: String,
    },
    PatIdentSubPat {
        ident: String,
    },
    // TODO: include closure arguments as well
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
    // TODO: Special case the unit tuple
    // TODO: watch out for tuple struct/variants with no fields
    PatTuple {
        bindings: Option<BindingOf>,
    },
    PatTupleStruct {
        bindings: Option<BindingOf>,
    },
    PatWild,
    PathLeadingColon,
    PathSegmentSelf,
    // TODO: reference the implementing type in an impl Block
    PathSegmentSelfType,
    PathSegmentCrate,
    // TODO: reference the actual module if it is inline, or refer to the relevant ancestor
    PathSegmentSuper,
    QSelf,
    QSelfAsTrait,
    ReceiverPath {
        method: Option<String>,
    },
    Attribute {
        outer: bool,
    },
    ItemExternCrate,
    ItemFn,
    ItemInlineMod,
    ItemExternMod,
    Unknown,
    // TODO: refine the detection of methods vs. associated functions
    FnToken {
        of: FnOf,
        name: String,
    },
    // TODO: refine the detection of methods vs. associated functions
    TraitItemMethod,
    // TODO: refine the detection of methods vs. associated functions
    ImplItemMethod,
    AsRename,
    AsRenameExternCrate,
    AsCast,
    AsyncFn,
    ExprArray,
    ExprArraySlice,
    ExprAsync,
    ExprAsyncMove,
    ExprAwait,
    ExprBreak {
        label: Option<String>,
        expr: bool,
    },
    ExprClosure,
    ExprClosureArguments,
    ExprClosureAsync,
    ExprClosureMove,
    ExprClosureStatic,
    ExprContinue {
        label: Option<String>,
    },
    ExprForLoopToken,
    ForLoopLocal {
        mutability: bool,
        ident: Option<String>,
    },
    // TODO: could we special case an `if` expression to see if it's being used as an expression vs. statement?
    // What about other expressions (`match`, `loop`, etc)?
    ExprIf,
    ExprIfLet,
    Else,
    ExprIndex {
        range: bool,
    },
    ImplItemConst,
    TraitItemConst,
    ItemConst,
    ConstParam,
    ConstFn,
    VisPublic,
    VisCrate,
    VisRestricted,
    WhereClause,
    Variant {
        name: String,
        fields: Option<Fields>,
    },
    VariantDiscriminant {
        name: String,
    },
    ItemEnum {
        empty: bool,
    },
    ItemForeignModAbi,
    FnAbi,
    BoundLifetimes,
    BoundLifetimesTraitBound {
        lifetime: String,
        ty: String,
        multiple: bool,
    },
    BoundLifetimesBareFnType,
    ItemImpl {
        trait_: bool,
    },
    ItemImplForTrait,
    ItemMacroRules {
        name: String,
    },
    TypeImplTrait,
    // TODO: maybe list the locals introduced, special case for when no locals are introduced
    Local {
        ident: Option<String>,
        mutability: bool,
    },
    ExprLoopToken,
    Label {
        loop_of: LoopOf,
    },
    True,
    False,
    // TODO: byte literals, escapes in chars and strings
    LitByteStr,
    LitChar,
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
    ExprMatchToken,
    ArmIfGuard,
    // TODO: handle special self cases, and explicit self references (`self: &Self`)
    MutSelf,
    ValueSelf {
        mutability: bool,
    },
    RefSelf,
    StaticMut,
    Static,
    // TODO: handle special cases: dyn
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
    UseGroupSelf {
        parent: String,
    },
    ExprReference {
        mutable: bool,
    },
    ExprRepeat {
        len: String,
    },
    ExprReturn,
    // TODO: handle ambiguity with struct variant instantiation. Also, in struct patterns.
    // Also in tuple structs.
    ExprStruct,
    ExprStructRest,
    ExprTryQuestionMark,
    ExprTryBlock,
    // TODO: explain 1-element tuple disambiguation (also in type)
    ExprTuple,
    ExprUnitTuple,
    ExprType,
    ExprUnsafe,
    ExprWhileLet,
    ExprWhile,
    ExprYield,
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
    ItemTraitSupertraits,
    ItemType,
    ItemUnion,
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
    KnownTypeU8,
    KnownTypeU16,
    KnownTypeU32,
    KnownTypeU64,
    KnownTypeU128,
    KnownTypeUSize,
    KnownTypeI8,
    KnownTypeI16,
    KnownTypeI32,
    KnownTypeI64,
    KnownTypeI128,
    KnownTypeISize,
    KnownTypeChar,
    KnownTypeBool,
    KnownTypeF32,
    KnownTypeF64,
    KnownTypeStr,
    KnownTypeStrSlice {
        mutability: bool,
    },
    TypeConstPtr,
    TypeMutPtr,
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
    Shebang,
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
        #[serde(rename(serialize = "function trait"))]
        FnTrait,
        Closure,
    }
];

variant![
    pub enum LoopOf {
        Loop,
        Block,
        While,
        #[serde(rename(serialize = "while let"))]
        WhileLet,
        For,
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
