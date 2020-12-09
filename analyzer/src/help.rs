#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "dev")]
use std::fmt::Debug;

std::thread_local! {
    static TEMPLATE: tinytemplate::TinyTemplate<'static> = {
        let mut template = init_template();
        template.add_formatter("generics", |value, bf| {
            render_generics(value, bf);
            Ok(())
        });
        template
    };
}

struct HelpData {
    template: &'static str,
    title: &'static str,
    book: Option<&'static str>,
    keyword: Option<&'static str>,
    std: Option<&'static str>,
}

std::include!(concat!(env!("OUT_DIR"), "/help.rs"));

// TODO: known conflicts/bugs
// * Clicking on an unnamed field in a struct/enum results in a clash between help for the type
//  and help for unnamed fields
// * `Self { x: 1 }` is not identified as a struct instantiation expression (probably same with patterns)
// * We need separate notions for "hitbox" and "highlight zone", so the user can hover over a token and
// see that something is clickable, but then highlight the relevant extent of the help

#[cfg_attr(all(not(test), not(feature = "dev")), derive(Serialize))]
#[cfg_attr(test, derive(Debug, Clone, Serialize, Deserialize, PartialEq))]
#[cfg_attr(feature = "dev", derive(Debug, Clone, Serialize))]
#[serde(tag = "type")]
pub enum HelpItem {
    Unknown,
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
    Binding {
        ident: String,
    },
    ExprArray,
    ExprArraySlice,
    ExprAssign,
    ExprAssignOp,
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
    ExprUnnamedField,
    ExprForLoopToken,
    ForLoopLocal {
        mutability: bool,
        ident: Option<String>,
    },
    // TODO: could we special case an `if` expression to see if it's being used as an expression vs. statement?
    // What about other expressions (`match`, `loop`, etc)?
    // Maybe explain that the test does not need parenthesis?
    ExprIf,
    // TODO: if-let and while-let can also introduce bindings
    ExprIfLet,
    Else,
    ExprIndex {
        range: bool,
    },
    ExprLoopToken,
    ExprMatchToken,
    ExprRangeHalfOpen {
        from: bool,
        to: bool,
    },
    ExprRangeClosed {
        from: bool,
        to: bool,
    },
    ExprReference {
        mutable: bool,
    },
    ExprRepeat {
        len: String,
    },
    ExprReturn {
        of: ReturnOf,
    },
    ExprStruct,
    ExprStructRest,
    ExprTryQuestionMark,
    ExprTryBlock,
    ExprTuple {
        single_comma: bool,
    },
    ExprUnitTuple,
    ExprType,
    ExprUnsafe,
    ExprWhileLet,
    ExprWhile,
    ExprYield,
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
    // TODO: watch out for tuple struct/variants with no fields
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=005c87b573205fe29993d051eebbd011
    PatUnit,
    PatTuple {
        bindings: Option<BindingOf>,
        single_comma: bool,
    },
    PatTupleStruct {
        bindings: Option<BindingOf>,
    },
    PatWild {
        last_arm: bool,
    },
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
    // TODO:
    // * outer top-level or module-level attributes are normal-ish
    // * outer attributes in other items are kind of weird
    // * explicit #[doc] attributes
    // * Other known attributes
    // * Maybe name of the item...?
    Attribute {
        outer: bool,
        known: Option<KnownAttribute>,
    },
    ItemExternCrate,
    ItemFn,
    ItemInlineMod,
    ItemExternMod,
    TraitItemMethod {
        of: FnOf,
        default: bool,
        trait_: String,
    },
    ImplItemMethod {
        of: FnOf,
        self_ty: String,
        trait_: Option<String>,
    },
    AsRename,
    // TODO: special-case "as _"
    AsRenameExternCrate,
    AsCast,
    AsyncFn,
    ImplItemConst,
    TraitItemConst,
    ItemConst,
    ConstParam {
        name: String,
        of: GenericsOf,
        of_name: String
    },
    // TODO: fallback for retro-compatibility, remove when confident
    ConstParamSimple,
    ConstFn,
    // TODO: specify `field` or `item` for visibility
    VisPublic,
    VisCrate,
    VisRestricted {
        path: VisRestrictedPath,
        in_: bool,
    },
    WhereClause,
    Variant {
        name: String,
        fields: Option<Fields>,
    },
    VariantDiscriminant {
        name: String,
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
        // TODO: negative impls should be more visible, in the bang symbol itself
        negative: bool,
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
    Label {
        loop_of: LoopOf,
    },
    True,
    False,
    // TODO: escapes in chars and strings
    LitByte,
    LitByteStr {
        raw: bool,
        prefix: Option<String>,
    },
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
    LitStr {
        raw: bool,
        prefix: Option<String>,
    },
    ArmIfGuard,
    MutSelf {
        explicit: bool,
        mutability: bool,
    },
    ValueSelf {
        mutability: bool,
        explicit: bool,
    },
    // TODO: maybe handle explicit references to `Self` type?
    // e.g.
    // struct Foo;
    // impl Foo {
    //   fn foo(self: Foo) {}
    // }
    SpecialSelf {
        mutability: bool,
    },
    RefSelf {
        explicit: bool,
        mutability: bool,
    },
    StaticMut,
    Static,
    // TODO: handle special cases: &dyn Foo, &(dyn Foo)
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
    ForeignItemType,
    RawIdent,
    ImplItemType,
    ItemUnsafeImpl,
    ItemEnum {
        empty: bool,
        generic: bool,
    },
    ItemStruct {
        unit: bool,
        name: String,
        generic: bool,
    },
    ItemAutoTrait,
    ItemUnsafeTrait,
    ItemTrait {
        generic: bool,
    },
    ItemTraitSupertraits,
    ItemType,
    ItemUnion {
        generic: bool,
    },
    ItemUse,
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
    TypeParam {
        name: String,
        of: GenericsOf,
        of_name: String
    },
    TypeParamUse,
    TypeParamBoundAdd,
    TypeTupleUnit,
    TypeTuple {
        single_comma: bool,
    },
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
    FieldUnnamedValue,
    Shebang,
    FatArrow,
    // TODO:
    //  * special-case module-level doc-comments
    //  * fix #[doc(foobar = "blah")]
    //  * elaborate on block vs. line comments
    DocBlock {
        outer: bool,
    },
    RArrow {
        return_of: ReturnOf,
    },
    StaticLifetime,
    LifetimeParam {
        of: GenericsOf,
        name: String,
        of_name: String,
    },
    Comment {
        block: bool,
    },
}

macro_rules! help_data {
    ($item:item) => {
        #[cfg_attr(all(not(test), not(feature = "dev")), derive(Serialize, Copy, Clone))]
        #[cfg_attr(test, derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq))]
        #[cfg_attr(feature = "dev", derive(Serialize, Copy, Clone, Debug))]
        #[serde(rename_all(serialize = "lowercase"))]
        $item
    };
}

help_data![
    pub enum FieldOf {
        #[serde(rename(serialize = "enum variant"))]
        Variant,
        Union,
        Struct,
    }
];

help_data![
    pub enum BindingOf {
        Let,
        Arg,
        ForLoop,
    }
];

help_data![
    pub enum IntMode {
        Binary,
        Hexadecimal,
        Octal,
    }
];

help_data![
    pub enum RestOf {
        Struct,
        Tuple,
        #[serde(rename(serialize = "tuple struct"))]
        TupleStruct,
    }
];

help_data![
    pub enum FnOf {
        Method,
        #[serde(rename(serialize = "associated function"))]
        AssociatedFunction,
    }
];

help_data![
    pub enum Fields {
        Named,
        Unnamed,
    }
];

help_data![
    pub enum ReturnOf {
        Function,
        Method,
        #[serde(rename(serialize = "bare function type"))]
        BareFunctionType,
        FnTrait,
        Closure,
        #[serde(rename(serialize = "async block"))]
        AsyncBlock,
    }
];

help_data![
    pub enum LoopOf {
        Loop,
        Block,
        While,
        #[serde(rename(serialize = "while let"))]
        WhileLet,
        For,
    }
];

help_data![
    pub enum VisRestrictedPath {
        Crate,
        Super,
        #[serde(rename(serialize = "self"))]
        Self_,
        Path,
    }
];

help_data![
    pub enum KnownAttribute {
        Doc,
    }
];

help_data![
    pub struct Generics {
        pub type_: bool,
        pub lifetime: bool,
        pub const_: bool,
        pub has_lifetime: bool,
        pub has_const: bool,
        pub of: GenericsOf,
    }
];

help_data![
    pub enum GenericsOf {
        #[serde(rename(serialize = "struct"))]
        Struct,
        #[serde(rename(serialize = "trait"))]
        Trait,
        #[serde(rename(serialize = "union"))]
        Union,
        #[serde(rename(serialize = "enum"))]
        Enum,
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

fn render_generics(value: &serde_json::Value, buffer: &mut String) {
    let lifetime = value
        .get("lifetime")
        .and_then(|val| val.as_bool())
        .unwrap_or(false);
    let type_ = value
        .get("type_")
        .and_then(|val| val.as_bool())
        .unwrap_or(false);
    let const_ = value
        .get("const_")
        .and_then(|val| val.as_bool())
        .unwrap_or(false);
    let of = value.get("of").and_then(|val| val.as_str()).unwrap_or("");

    buffer.push_str("This ");
    buffer.push_str(of);
    buffer.push_str(" is _generic_ ");

    // if type_ {
    //     buffer.push_str("which means it can be _instantiated_ for particular values of ")
    // }

    // buffer.push_str(context);
}

fn on_add_template<E>(result: Result<(), E>) {
    #[cfg(features = "dev")]
    {
        result.unwrap();
    }
    #[cfg(not(features = "dev"))]
    {
        let _ = result;
    }
}
