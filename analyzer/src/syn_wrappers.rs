use crate::ir::Range;

#[derive(Clone, Debug)]
pub struct Comment {
    pub block: bool,
    /// None -> normal, Some(inner) -> doc
    pub doc: Option<bool>,
    pub range: Range,
}

#[derive(Clone, Copy)]
pub enum Syn<'a> {
    Abi(&'a syn::Abi),
    AngleBracketedGenericArguments(&'a syn::AngleBracketedGenericArguments),
    Arm(&'a syn::Arm),
    AttrStyle(&'a syn::AttrStyle),
    Attribute(&'a syn::Attribute),
    BareFnArg(&'a syn::BareFnArg),
    BinOp(&'a syn::BinOp),
    Binding(&'a syn::Binding),
    Block(&'a syn::Block),
    BoundLifetimes(&'a syn::BoundLifetimes),
    ConstParam(&'a syn::ConstParam),
    Constraint(&'a syn::Constraint),
    Expr(&'a syn::Expr),
    ExprArray(&'a syn::ExprArray),
    ExprAssign(&'a syn::ExprAssign),
    ExprAssignOp(&'a syn::ExprAssignOp),
    ExprAsync(&'a syn::ExprAsync),
    ExprAwait(&'a syn::ExprAwait),
    ExprBinary(&'a syn::ExprBinary),
    ExprBlock(&'a syn::ExprBlock),
    ExprBox(&'a syn::ExprBox),
    ExprBreak(&'a syn::ExprBreak),
    ExprCall(&'a syn::ExprCall),
    ExprCast(&'a syn::ExprCast),
    ExprClosure(&'a syn::ExprClosure),
    ExprContinue(&'a syn::ExprContinue),
    ExprField(&'a syn::ExprField),
    ExprForLoop(&'a syn::ExprForLoop),
    ExprGroup(&'a syn::ExprGroup),
    ExprIf(&'a syn::ExprIf),
    ExprIndex(&'a syn::ExprIndex),
    ExprLet(&'a syn::ExprLet),
    ExprLit(&'a syn::ExprLit),
    ExprLoop(&'a syn::ExprLoop),
    ExprMacro(&'a syn::ExprMacro),
    ExprMatch(&'a syn::ExprMatch),
    ExprMethodCall(&'a syn::ExprMethodCall),
    ExprParen(&'a syn::ExprParen),
    ExprPath(&'a syn::ExprPath),
    ExprRange(&'a syn::ExprRange),
    ExprReference(&'a syn::ExprReference),
    ExprRepeat(&'a syn::ExprRepeat),
    ExprReturn(&'a syn::ExprReturn),
    ExprStruct(&'a syn::ExprStruct),
    ExprTry(&'a syn::ExprTry),
    ExprTryBlock(&'a syn::ExprTryBlock),
    ExprTuple(&'a syn::ExprTuple),
    ExprType(&'a syn::ExprType),
    ExprUnary(&'a syn::ExprUnary),
    ExprUnsafe(&'a syn::ExprUnsafe),
    ExprWhile(&'a syn::ExprWhile),
    ExprYield(&'a syn::ExprYield),
    Field(&'a syn::Field),
    FieldPat(&'a syn::FieldPat),
    FieldValue(&'a syn::FieldValue),
    Fields(&'a syn::Fields),
    FieldsNamed(&'a syn::FieldsNamed),
    FieldsUnnamed(&'a syn::FieldsUnnamed),
    File(&'a syn::File),
    FnArg(&'a syn::FnArg),
    ForeignItem(&'a syn::ForeignItem),
    ForeignItemFn(&'a syn::ForeignItemFn),
    ForeignItemMacro(&'a syn::ForeignItemMacro),
    ForeignItemStatic(&'a syn::ForeignItemStatic),
    ForeignItemType(&'a syn::ForeignItemType),
    GenericArgument(&'a syn::GenericArgument),
    GenericMethodArgument(&'a syn::GenericMethodArgument),
    GenericParam(&'a syn::GenericParam),
    Generics(&'a syn::Generics),
    Ident(&'a proc_macro2::Ident),
    ImplItem(&'a syn::ImplItem),
    ImplItemConst(&'a syn::ImplItemConst),
    ImplItemMacro(&'a syn::ImplItemMacro),
    ImplItemMethod(&'a syn::ImplItemMethod),
    ImplItemType(&'a syn::ImplItemType),
    Index(&'a syn::Index),
    Item(&'a syn::Item),
    ItemConst(&'a syn::ItemConst),
    ItemEnum(&'a syn::ItemEnum),
    ItemExternCrate(&'a syn::ItemExternCrate),
    ItemFn(&'a syn::ItemFn),
    ItemForeignMod(&'a syn::ItemForeignMod),
    ItemImpl(&'a syn::ItemImpl),
    ItemMacro(&'a syn::ItemMacro),
    ItemMacro2(&'a syn::ItemMacro2),
    ItemMod(&'a syn::ItemMod),
    ItemStatic(&'a syn::ItemStatic),
    ItemStruct(&'a syn::ItemStruct),
    ItemTrait(&'a syn::ItemTrait),
    ItemTraitAlias(&'a syn::ItemTraitAlias),
    ItemType(&'a syn::ItemType),
    ItemUnion(&'a syn::ItemUnion),
    ItemUse(&'a syn::ItemUse),
    Label(&'a syn::Label),
    Lifetime(&'a syn::Lifetime),
    LifetimeDef(&'a syn::LifetimeDef),
    Lit(&'a syn::Lit),
    LitBool(&'a syn::LitBool),
    LitByte(&'a syn::LitByte),
    LitByteStr(&'a syn::LitByteStr),
    LitChar(&'a syn::LitChar),
    LitFloat(&'a syn::LitFloat),
    LitInt(&'a syn::LitInt),
    LitStr(&'a syn::LitStr),
    Local(&'a syn::Local),
    Macro(&'a syn::Macro),
    Member(&'a syn::Member),
    MethodTurbofish(&'a syn::MethodTurbofish),
    ParenthesizedGenericArguments(&'a syn::ParenthesizedGenericArguments),
    Pat(&'a syn::Pat),
    PatBox(&'a syn::PatBox),
    PatIdent(&'a syn::PatIdent),
    PatLit(&'a syn::PatLit),
    PatMacro(&'a syn::PatMacro),
    PatOr(&'a syn::PatOr),
    PatPath(&'a syn::PatPath),
    PatRange(&'a syn::PatRange),
    PatReference(&'a syn::PatReference),
    PatRest(&'a syn::PatRest),
    PatSlice(&'a syn::PatSlice),
    PatStruct(&'a syn::PatStruct),
    PatTuple(&'a syn::PatTuple),
    PatTupleStruct(&'a syn::PatTupleStruct),
    PatType(&'a syn::PatType),
    PatWild(&'a syn::PatWild),
    Path(&'a syn::Path),
    PathArguments(&'a syn::PathArguments),
    PathSegment(&'a syn::PathSegment),
    PredicateEq(&'a syn::PredicateEq),
    PredicateLifetime(&'a syn::PredicateLifetime),
    PredicateType(&'a syn::PredicateType),
    QSelf(&'a syn::QSelf),
    Receiver(&'a syn::Receiver),
    ReturnType(&'a syn::ReturnType),
    Signature(&'a syn::Signature),
    Stmt(&'a syn::Stmt),
    TraitBound(&'a syn::TraitBound),
    TraitBoundModifier(&'a syn::TraitBoundModifier),
    TraitItem(&'a syn::TraitItem),
    TraitItemConst(&'a syn::TraitItemConst),
    TraitItemMacro(&'a syn::TraitItemMacro),
    TraitItemMethod(&'a syn::TraitItemMethod),
    TraitItemType(&'a syn::TraitItemType),
    Type(&'a syn::Type),
    TypeArray(&'a syn::TypeArray),
    TypeBareFn(&'a syn::TypeBareFn),
    TypeGroup(&'a syn::TypeGroup),
    TypeImplTrait(&'a syn::TypeImplTrait),
    TypeInfer(&'a syn::TypeInfer),
    TypeMacro(&'a syn::TypeMacro),
    TypeNever(&'a syn::TypeNever),
    TypeParam(&'a syn::TypeParam),
    TypeParamBound(&'a syn::TypeParamBound),
    TypeParen(&'a syn::TypeParen),
    TypePath(&'a syn::TypePath),
    TypePtr(&'a syn::TypePtr),
    TypeReference(&'a syn::TypeReference),
    TypeSlice(&'a syn::TypeSlice),
    TypeTraitObject(&'a syn::TypeTraitObject),
    TypeTuple(&'a syn::TypeTuple),
    UnOp(&'a syn::UnOp),
    UseGlob(&'a syn::UseGlob),
    UseGroup(&'a syn::UseGroup),
    UseName(&'a syn::UseName),
    UsePath(&'a syn::UsePath),
    UseRename(&'a syn::UseRename),
    UseTree(&'a syn::UseTree),
    Variadic(&'a syn::Variadic),
    Variant(&'a syn::Variant),
    VisCrate(&'a syn::VisCrate),
    VisPublic(&'a syn::VisPublic),
    VisRestricted(&'a syn::VisRestricted),
    Visibility(&'a syn::Visibility),
    WhereClause(&'a syn::WhereClause),
    WherePredicate(&'a syn::WherePredicate),
    // This is a fake node that does not actually come from Syn
    Comment(&'a Comment),
}

impl<'a> Syn<'a> {
    pub fn data(&self) -> *const () {
        match self {
            Syn::Abi(node) => *node as *const _ as *const (),
            Syn::AngleBracketedGenericArguments(node) => *node as *const _ as *const (),
            Syn::Arm(node) => *node as *const _ as *const (),
            Syn::AttrStyle(node) => *node as *const _ as *const (),
            Syn::Attribute(node) => *node as *const _ as *const (),
            Syn::BareFnArg(node) => *node as *const _ as *const (),
            Syn::BinOp(node) => *node as *const _ as *const (),
            Syn::Binding(node) => *node as *const _ as *const (),
            Syn::Block(node) => *node as *const _ as *const (),
            Syn::BoundLifetimes(node) => *node as *const _ as *const (),
            Syn::ConstParam(node) => *node as *const _ as *const (),
            Syn::Constraint(node) => *node as *const _ as *const (),
            Syn::Expr(node) => *node as *const _ as *const (),
            Syn::ExprArray(node) => *node as *const _ as *const (),
            Syn::ExprAssign(node) => *node as *const _ as *const (),
            Syn::ExprAssignOp(node) => *node as *const _ as *const (),
            Syn::ExprAsync(node) => *node as *const _ as *const (),
            Syn::ExprAwait(node) => *node as *const _ as *const (),
            Syn::ExprBinary(node) => *node as *const _ as *const (),
            Syn::ExprBlock(node) => *node as *const _ as *const (),
            Syn::ExprBox(node) => *node as *const _ as *const (),
            Syn::ExprBreak(node) => *node as *const _ as *const (),
            Syn::ExprCall(node) => *node as *const _ as *const (),
            Syn::ExprCast(node) => *node as *const _ as *const (),
            Syn::ExprClosure(node) => *node as *const _ as *const (),
            Syn::ExprContinue(node) => *node as *const _ as *const (),
            Syn::ExprField(node) => *node as *const _ as *const (),
            Syn::ExprForLoop(node) => *node as *const _ as *const (),
            Syn::ExprGroup(node) => *node as *const _ as *const (),
            Syn::ExprIf(node) => *node as *const _ as *const (),
            Syn::ExprIndex(node) => *node as *const _ as *const (),
            Syn::ExprLet(node) => *node as *const _ as *const (),
            Syn::ExprLit(node) => *node as *const _ as *const (),
            Syn::ExprLoop(node) => *node as *const _ as *const (),
            Syn::ExprMacro(node) => *node as *const _ as *const (),
            Syn::ExprMatch(node) => *node as *const _ as *const (),
            Syn::ExprMethodCall(node) => *node as *const _ as *const (),
            Syn::ExprParen(node) => *node as *const _ as *const (),
            Syn::ExprPath(node) => *node as *const _ as *const (),
            Syn::ExprRange(node) => *node as *const _ as *const (),
            Syn::ExprReference(node) => *node as *const _ as *const (),
            Syn::ExprRepeat(node) => *node as *const _ as *const (),
            Syn::ExprReturn(node) => *node as *const _ as *const (),
            Syn::ExprStruct(node) => *node as *const _ as *const (),
            Syn::ExprTry(node) => *node as *const _ as *const (),
            Syn::ExprTryBlock(node) => *node as *const _ as *const (),
            Syn::ExprTuple(node) => *node as *const _ as *const (),
            Syn::ExprType(node) => *node as *const _ as *const (),
            Syn::ExprUnary(node) => *node as *const _ as *const (),
            Syn::ExprUnsafe(node) => *node as *const _ as *const (),
            Syn::ExprWhile(node) => *node as *const _ as *const (),
            Syn::ExprYield(node) => *node as *const _ as *const (),
            Syn::Field(node) => *node as *const _ as *const (),
            Syn::FieldPat(node) => *node as *const _ as *const (),
            Syn::FieldValue(node) => *node as *const _ as *const (),
            Syn::Fields(node) => *node as *const _ as *const (),
            Syn::FieldsNamed(node) => *node as *const _ as *const (),
            Syn::FieldsUnnamed(node) => *node as *const _ as *const (),
            Syn::File(node) => *node as *const _ as *const (),
            Syn::FnArg(node) => *node as *const _ as *const (),
            Syn::ForeignItem(node) => *node as *const _ as *const (),
            Syn::ForeignItemFn(node) => *node as *const _ as *const (),
            Syn::ForeignItemMacro(node) => *node as *const _ as *const (),
            Syn::ForeignItemStatic(node) => *node as *const _ as *const (),
            Syn::ForeignItemType(node) => *node as *const _ as *const (),
            Syn::GenericArgument(node) => *node as *const _ as *const (),
            Syn::GenericMethodArgument(node) => *node as *const _ as *const (),
            Syn::GenericParam(node) => *node as *const _ as *const (),
            Syn::Generics(node) => *node as *const _ as *const (),
            Syn::Ident(node) => *node as *const _ as *const (),
            Syn::ImplItem(node) => *node as *const _ as *const (),
            Syn::ImplItemConst(node) => *node as *const _ as *const (),
            Syn::ImplItemMacro(node) => *node as *const _ as *const (),
            Syn::ImplItemMethod(node) => *node as *const _ as *const (),
            Syn::ImplItemType(node) => *node as *const _ as *const (),
            Syn::Index(node) => *node as *const _ as *const (),
            Syn::Item(node) => *node as *const _ as *const (),
            Syn::ItemConst(node) => *node as *const _ as *const (),
            Syn::ItemEnum(node) => *node as *const _ as *const (),
            Syn::ItemExternCrate(node) => *node as *const _ as *const (),
            Syn::ItemFn(node) => *node as *const _ as *const (),
            Syn::ItemForeignMod(node) => *node as *const _ as *const (),
            Syn::ItemImpl(node) => *node as *const _ as *const (),
            Syn::ItemMacro(node) => *node as *const _ as *const (),
            Syn::ItemMacro2(node) => *node as *const _ as *const (),
            Syn::ItemMod(node) => *node as *const _ as *const (),
            Syn::ItemStatic(node) => *node as *const _ as *const (),
            Syn::ItemStruct(node) => *node as *const _ as *const (),
            Syn::ItemTrait(node) => *node as *const _ as *const (),
            Syn::ItemTraitAlias(node) => *node as *const _ as *const (),
            Syn::ItemType(node) => *node as *const _ as *const (),
            Syn::ItemUnion(node) => *node as *const _ as *const (),
            Syn::ItemUse(node) => *node as *const _ as *const (),
            Syn::Label(node) => *node as *const _ as *const (),
            Syn::Lifetime(node) => *node as *const _ as *const (),
            Syn::LifetimeDef(node) => *node as *const _ as *const (),
            Syn::Lit(node) => *node as *const _ as *const (),
            Syn::LitBool(node) => *node as *const _ as *const (),
            Syn::LitByte(node) => *node as *const _ as *const (),
            Syn::LitByteStr(node) => *node as *const _ as *const (),
            Syn::LitChar(node) => *node as *const _ as *const (),
            Syn::LitFloat(node) => *node as *const _ as *const (),
            Syn::LitInt(node) => *node as *const _ as *const (),
            Syn::LitStr(node) => *node as *const _ as *const (),
            Syn::Local(node) => *node as *const _ as *const (),
            Syn::Macro(node) => *node as *const _ as *const (),
            Syn::Member(node) => *node as *const _ as *const (),
            Syn::MethodTurbofish(node) => *node as *const _ as *const (),
            Syn::ParenthesizedGenericArguments(node) => *node as *const _ as *const (),
            Syn::Pat(node) => *node as *const _ as *const (),
            Syn::PatBox(node) => *node as *const _ as *const (),
            Syn::PatIdent(node) => *node as *const _ as *const (),
            Syn::PatLit(node) => *node as *const _ as *const (),
            Syn::PatMacro(node) => *node as *const _ as *const (),
            Syn::PatOr(node) => *node as *const _ as *const (),
            Syn::PatPath(node) => *node as *const _ as *const (),
            Syn::PatRange(node) => *node as *const _ as *const (),
            Syn::PatReference(node) => *node as *const _ as *const (),
            Syn::PatRest(node) => *node as *const _ as *const (),
            Syn::PatSlice(node) => *node as *const _ as *const (),
            Syn::PatStruct(node) => *node as *const _ as *const (),
            Syn::PatTuple(node) => *node as *const _ as *const (),
            Syn::PatTupleStruct(node) => *node as *const _ as *const (),
            Syn::PatType(node) => *node as *const _ as *const (),
            Syn::PatWild(node) => *node as *const _ as *const (),
            Syn::Path(node) => *node as *const _ as *const (),
            Syn::PathArguments(node) => *node as *const _ as *const (),
            Syn::PathSegment(node) => *node as *const _ as *const (),
            Syn::PredicateEq(node) => *node as *const _ as *const (),
            Syn::PredicateLifetime(node) => *node as *const _ as *const (),
            Syn::PredicateType(node) => *node as *const _ as *const (),
            Syn::QSelf(node) => *node as *const _ as *const (),
            Syn::Receiver(node) => *node as *const _ as *const (),
            Syn::ReturnType(node) => *node as *const _ as *const (),
            Syn::Signature(node) => *node as *const _ as *const (),
            Syn::Stmt(node) => *node as *const _ as *const (),
            Syn::TraitBound(node) => *node as *const _ as *const (),
            Syn::TraitBoundModifier(node) => *node as *const _ as *const (),
            Syn::TraitItem(node) => *node as *const _ as *const (),
            Syn::TraitItemConst(node) => *node as *const _ as *const (),
            Syn::TraitItemMacro(node) => *node as *const _ as *const (),
            Syn::TraitItemMethod(node) => *node as *const _ as *const (),
            Syn::TraitItemType(node) => *node as *const _ as *const (),
            Syn::Type(node) => *node as *const _ as *const (),
            Syn::TypeArray(node) => *node as *const _ as *const (),
            Syn::TypeBareFn(node) => *node as *const _ as *const (),
            Syn::TypeGroup(node) => *node as *const _ as *const (),
            Syn::TypeImplTrait(node) => *node as *const _ as *const (),
            Syn::TypeInfer(node) => *node as *const _ as *const (),
            Syn::TypeMacro(node) => *node as *const _ as *const (),
            Syn::TypeNever(node) => *node as *const _ as *const (),
            Syn::TypeParam(node) => *node as *const _ as *const (),
            Syn::TypeParamBound(node) => *node as *const _ as *const (),
            Syn::TypeParen(node) => *node as *const _ as *const (),
            Syn::TypePath(node) => *node as *const _ as *const (),
            Syn::TypePtr(node) => *node as *const _ as *const (),
            Syn::TypeReference(node) => *node as *const _ as *const (),
            Syn::TypeSlice(node) => *node as *const _ as *const (),
            Syn::TypeTraitObject(node) => *node as *const _ as *const (),
            Syn::TypeTuple(node) => *node as *const _ as *const (),
            Syn::UnOp(node) => *node as *const _ as *const (),
            Syn::UseGlob(node) => *node as *const _ as *const (),
            Syn::UseGroup(node) => *node as *const _ as *const (),
            Syn::UseName(node) => *node as *const _ as *const (),
            Syn::UsePath(node) => *node as *const _ as *const (),
            Syn::UseRename(node) => *node as *const _ as *const (),
            Syn::UseTree(node) => *node as *const _ as *const (),
            Syn::Variadic(node) => *node as *const _ as *const (),
            Syn::Variant(node) => *node as *const _ as *const (),
            Syn::VisCrate(node) => *node as *const _ as *const (),
            Syn::VisPublic(node) => *node as *const _ as *const (),
            Syn::VisRestricted(node) => *node as *const _ as *const (),
            Syn::Visibility(node) => *node as *const _ as *const (),
            Syn::WhereClause(node) => *node as *const _ as *const (),
            Syn::WherePredicate(node) => *node as *const _ as *const (),
            Syn::Comment(node) => *node as *const _ as *const (),
        }
    }

    pub fn kind(&self) -> SynKind {
        self.into()
    }

    pub unsafe fn from_raw(data: *const (), kind: SynKind) -> Syn<'a> {
        match kind {
            SynKind::Abi => Syn::Abi(&*(data as *const _)),
            SynKind::AngleBracketedGenericArguments => {
                Syn::AngleBracketedGenericArguments(&*(data as *const _))
            }
            SynKind::Arm => Syn::Arm(&*(data as *const _)),
            SynKind::AttrStyle => Syn::AttrStyle(&*(data as *const _)),
            SynKind::Attribute => Syn::Attribute(&*(data as *const _)),
            SynKind::BareFnArg => Syn::BareFnArg(&*(data as *const _)),
            SynKind::BinOp => Syn::BinOp(&*(data as *const _)),
            SynKind::Binding => Syn::Binding(&*(data as *const _)),
            SynKind::Block => Syn::Block(&*(data as *const _)),
            SynKind::BoundLifetimes => Syn::BoundLifetimes(&*(data as *const _)),
            SynKind::ConstParam => Syn::ConstParam(&*(data as *const _)),
            SynKind::Constraint => Syn::Constraint(&*(data as *const _)),
            SynKind::Expr => Syn::Expr(&*(data as *const _)),
            SynKind::ExprArray => Syn::ExprArray(&*(data as *const _)),
            SynKind::ExprAssign => Syn::ExprAssign(&*(data as *const _)),
            SynKind::ExprAssignOp => Syn::ExprAssignOp(&*(data as *const _)),
            SynKind::ExprAsync => Syn::ExprAsync(&*(data as *const _)),
            SynKind::ExprAwait => Syn::ExprAwait(&*(data as *const _)),
            SynKind::ExprBinary => Syn::ExprBinary(&*(data as *const _)),
            SynKind::ExprBlock => Syn::ExprBlock(&*(data as *const _)),
            SynKind::ExprBox => Syn::ExprBox(&*(data as *const _)),
            SynKind::ExprBreak => Syn::ExprBreak(&*(data as *const _)),
            SynKind::ExprCall => Syn::ExprCall(&*(data as *const _)),
            SynKind::ExprCast => Syn::ExprCast(&*(data as *const _)),
            SynKind::ExprClosure => Syn::ExprClosure(&*(data as *const _)),
            SynKind::ExprContinue => Syn::ExprContinue(&*(data as *const _)),
            SynKind::ExprField => Syn::ExprField(&*(data as *const _)),
            SynKind::ExprForLoop => Syn::ExprForLoop(&*(data as *const _)),
            SynKind::ExprGroup => Syn::ExprGroup(&*(data as *const _)),
            SynKind::ExprIf => Syn::ExprIf(&*(data as *const _)),
            SynKind::ExprIndex => Syn::ExprIndex(&*(data as *const _)),
            SynKind::ExprLet => Syn::ExprLet(&*(data as *const _)),
            SynKind::ExprLit => Syn::ExprLit(&*(data as *const _)),
            SynKind::ExprLoop => Syn::ExprLoop(&*(data as *const _)),
            SynKind::ExprMacro => Syn::ExprMacro(&*(data as *const _)),
            SynKind::ExprMatch => Syn::ExprMatch(&*(data as *const _)),
            SynKind::ExprMethodCall => Syn::ExprMethodCall(&*(data as *const _)),
            SynKind::ExprParen => Syn::ExprParen(&*(data as *const _)),
            SynKind::ExprPath => Syn::ExprPath(&*(data as *const _)),
            SynKind::ExprRange => Syn::ExprRange(&*(data as *const _)),
            SynKind::ExprReference => Syn::ExprReference(&*(data as *const _)),
            SynKind::ExprRepeat => Syn::ExprRepeat(&*(data as *const _)),
            SynKind::ExprReturn => Syn::ExprReturn(&*(data as *const _)),
            SynKind::ExprStruct => Syn::ExprStruct(&*(data as *const _)),
            SynKind::ExprTry => Syn::ExprTry(&*(data as *const _)),
            SynKind::ExprTryBlock => Syn::ExprTryBlock(&*(data as *const _)),
            SynKind::ExprTuple => Syn::ExprTuple(&*(data as *const _)),
            SynKind::ExprType => Syn::ExprType(&*(data as *const _)),
            SynKind::ExprUnary => Syn::ExprUnary(&*(data as *const _)),
            SynKind::ExprUnsafe => Syn::ExprUnsafe(&*(data as *const _)),
            SynKind::ExprWhile => Syn::ExprWhile(&*(data as *const _)),
            SynKind::ExprYield => Syn::ExprYield(&*(data as *const _)),
            SynKind::Field => Syn::Field(&*(data as *const _)),
            SynKind::FieldPat => Syn::FieldPat(&*(data as *const _)),
            SynKind::FieldValue => Syn::FieldValue(&*(data as *const _)),
            SynKind::Fields => Syn::Fields(&*(data as *const _)),
            SynKind::FieldsNamed => Syn::FieldsNamed(&*(data as *const _)),
            SynKind::FieldsUnnamed => Syn::FieldsUnnamed(&*(data as *const _)),
            SynKind::File => Syn::File(&*(data as *const _)),
            SynKind::FnArg => Syn::FnArg(&*(data as *const _)),
            SynKind::ForeignItem => Syn::ForeignItem(&*(data as *const _)),
            SynKind::ForeignItemFn => Syn::ForeignItemFn(&*(data as *const _)),
            SynKind::ForeignItemMacro => Syn::ForeignItemMacro(&*(data as *const _)),
            SynKind::ForeignItemStatic => Syn::ForeignItemStatic(&*(data as *const _)),
            SynKind::ForeignItemType => Syn::ForeignItemType(&*(data as *const _)),
            SynKind::GenericArgument => Syn::GenericArgument(&*(data as *const _)),
            SynKind::GenericMethodArgument => Syn::GenericMethodArgument(&*(data as *const _)),
            SynKind::GenericParam => Syn::GenericParam(&*(data as *const _)),
            SynKind::Generics => Syn::Generics(&*(data as *const _)),
            SynKind::Ident => Syn::Ident(&*(data as *const _)),
            SynKind::ImplItem => Syn::ImplItem(&*(data as *const _)),
            SynKind::ImplItemConst => Syn::ImplItemConst(&*(data as *const _)),
            SynKind::ImplItemMacro => Syn::ImplItemMacro(&*(data as *const _)),
            SynKind::ImplItemMethod => Syn::ImplItemMethod(&*(data as *const _)),
            SynKind::ImplItemType => Syn::ImplItemType(&*(data as *const _)),
            SynKind::Index => Syn::Index(&*(data as *const _)),
            SynKind::Item => Syn::Item(&*(data as *const _)),
            SynKind::ItemConst => Syn::ItemConst(&*(data as *const _)),
            SynKind::ItemEnum => Syn::ItemEnum(&*(data as *const _)),
            SynKind::ItemExternCrate => Syn::ItemExternCrate(&*(data as *const _)),
            SynKind::ItemFn => Syn::ItemFn(&*(data as *const _)),
            SynKind::ItemForeignMod => Syn::ItemForeignMod(&*(data as *const _)),
            SynKind::ItemImpl => Syn::ItemImpl(&*(data as *const _)),
            SynKind::ItemMacro => Syn::ItemMacro(&*(data as *const _)),
            SynKind::ItemMacro2 => Syn::ItemMacro2(&*(data as *const _)),
            SynKind::ItemMod => Syn::ItemMod(&*(data as *const _)),
            SynKind::ItemStatic => Syn::ItemStatic(&*(data as *const _)),
            SynKind::ItemStruct => Syn::ItemStruct(&*(data as *const _)),
            SynKind::ItemTrait => Syn::ItemTrait(&*(data as *const _)),
            SynKind::ItemTraitAlias => Syn::ItemTraitAlias(&*(data as *const _)),
            SynKind::ItemType => Syn::ItemType(&*(data as *const _)),
            SynKind::ItemUnion => Syn::ItemUnion(&*(data as *const _)),
            SynKind::ItemUse => Syn::ItemUse(&*(data as *const _)),
            SynKind::Label => Syn::Label(&*(data as *const _)),
            SynKind::Lifetime => Syn::Lifetime(&*(data as *const _)),
            SynKind::LifetimeDef => Syn::LifetimeDef(&*(data as *const _)),
            SynKind::Lit => Syn::Lit(&*(data as *const _)),
            SynKind::LitBool => Syn::LitBool(&*(data as *const _)),
            SynKind::LitByte => Syn::LitByte(&*(data as *const _)),
            SynKind::LitByteStr => Syn::LitByteStr(&*(data as *const _)),
            SynKind::LitChar => Syn::LitChar(&*(data as *const _)),
            SynKind::LitFloat => Syn::LitFloat(&*(data as *const _)),
            SynKind::LitInt => Syn::LitInt(&*(data as *const _)),
            SynKind::LitStr => Syn::LitStr(&*(data as *const _)),
            SynKind::Local => Syn::Local(&*(data as *const _)),
            SynKind::Macro => Syn::Macro(&*(data as *const _)),
            SynKind::Member => Syn::Member(&*(data as *const _)),
            SynKind::MethodTurbofish => Syn::MethodTurbofish(&*(data as *const _)),
            SynKind::ParenthesizedGenericArguments => {
                Syn::ParenthesizedGenericArguments(&*(data as *const _))
            }
            SynKind::Pat => Syn::Pat(&*(data as *const _)),
            SynKind::PatBox => Syn::PatBox(&*(data as *const _)),
            SynKind::PatIdent => Syn::PatIdent(&*(data as *const _)),
            SynKind::PatLit => Syn::PatLit(&*(data as *const _)),
            SynKind::PatMacro => Syn::PatMacro(&*(data as *const _)),
            SynKind::PatOr => Syn::PatOr(&*(data as *const _)),
            SynKind::PatPath => Syn::PatPath(&*(data as *const _)),
            SynKind::PatRange => Syn::PatRange(&*(data as *const _)),
            SynKind::PatReference => Syn::PatReference(&*(data as *const _)),
            SynKind::PatRest => Syn::PatRest(&*(data as *const _)),
            SynKind::PatSlice => Syn::PatSlice(&*(data as *const _)),
            SynKind::PatStruct => Syn::PatStruct(&*(data as *const _)),
            SynKind::PatTuple => Syn::PatTuple(&*(data as *const _)),
            SynKind::PatTupleStruct => Syn::PatTupleStruct(&*(data as *const _)),
            SynKind::PatType => Syn::PatType(&*(data as *const _)),
            SynKind::PatWild => Syn::PatWild(&*(data as *const _)),
            SynKind::Path => Syn::Path(&*(data as *const _)),
            SynKind::PathArguments => Syn::PathArguments(&*(data as *const _)),
            SynKind::PathSegment => Syn::PathSegment(&*(data as *const _)),
            SynKind::PredicateEq => Syn::PredicateEq(&*(data as *const _)),
            SynKind::PredicateLifetime => Syn::PredicateLifetime(&*(data as *const _)),
            SynKind::PredicateType => Syn::PredicateType(&*(data as *const _)),
            SynKind::QSelf => Syn::QSelf(&*(data as *const _)),
            SynKind::Receiver => Syn::Receiver(&*(data as *const _)),
            SynKind::ReturnType => Syn::ReturnType(&*(data as *const _)),
            SynKind::Signature => Syn::Signature(&*(data as *const _)),
            SynKind::Stmt => Syn::Stmt(&*(data as *const _)),
            SynKind::TraitBound => Syn::TraitBound(&*(data as *const _)),
            SynKind::TraitBoundModifier => Syn::TraitBoundModifier(&*(data as *const _)),
            SynKind::TraitItem => Syn::TraitItem(&*(data as *const _)),
            SynKind::TraitItemConst => Syn::TraitItemConst(&*(data as *const _)),
            SynKind::TraitItemMacro => Syn::TraitItemMacro(&*(data as *const _)),
            SynKind::TraitItemMethod => Syn::TraitItemMethod(&*(data as *const _)),
            SynKind::TraitItemType => Syn::TraitItemType(&*(data as *const _)),
            SynKind::Type => Syn::Type(&*(data as *const _)),
            SynKind::TypeArray => Syn::TypeArray(&*(data as *const _)),
            SynKind::TypeBareFn => Syn::TypeBareFn(&*(data as *const _)),
            SynKind::TypeGroup => Syn::TypeGroup(&*(data as *const _)),
            SynKind::TypeImplTrait => Syn::TypeImplTrait(&*(data as *const _)),
            SynKind::TypeInfer => Syn::TypeInfer(&*(data as *const _)),
            SynKind::TypeMacro => Syn::TypeMacro(&*(data as *const _)),
            SynKind::TypeNever => Syn::TypeNever(&*(data as *const _)),
            SynKind::TypeParam => Syn::TypeParam(&*(data as *const _)),
            SynKind::TypeParamBound => Syn::TypeParamBound(&*(data as *const _)),
            SynKind::TypeParen => Syn::TypeParen(&*(data as *const _)),
            SynKind::TypePath => Syn::TypePath(&*(data as *const _)),
            SynKind::TypePtr => Syn::TypePtr(&*(data as *const _)),
            SynKind::TypeReference => Syn::TypeReference(&*(data as *const _)),
            SynKind::TypeSlice => Syn::TypeSlice(&*(data as *const _)),
            SynKind::TypeTraitObject => Syn::TypeTraitObject(&*(data as *const _)),
            SynKind::TypeTuple => Syn::TypeTuple(&*(data as *const _)),
            SynKind::UnOp => Syn::UnOp(&*(data as *const _)),
            SynKind::UseGlob => Syn::UseGlob(&*(data as *const _)),
            SynKind::UseGroup => Syn::UseGroup(&*(data as *const _)),
            SynKind::UseName => Syn::UseName(&*(data as *const _)),
            SynKind::UsePath => Syn::UsePath(&*(data as *const _)),
            SynKind::UseRename => Syn::UseRename(&*(data as *const _)),
            SynKind::UseTree => Syn::UseTree(&*(data as *const _)),
            SynKind::Variadic => Syn::Variadic(&*(data as *const _)),
            SynKind::Variant => Syn::Variant(&*(data as *const _)),
            SynKind::VisCrate => Syn::VisCrate(&*(data as *const _)),
            SynKind::VisPublic => Syn::VisPublic(&*(data as *const _)),
            SynKind::VisRestricted => Syn::VisRestricted(&*(data as *const _)),
            SynKind::Visibility => Syn::Visibility(&*(data as *const _)),
            SynKind::WhereClause => Syn::WhereClause(&*(data as *const _)),
            SynKind::WherePredicate => Syn::WherePredicate(&*(data as *const _)),
            SynKind::Comment => Syn::Comment(&*(data as *const _)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SynKind {
    Abi,
    AngleBracketedGenericArguments,
    Arm,
    AttrStyle,
    Attribute,
    BareFnArg,
    BinOp,
    Binding,
    Block,
    BoundLifetimes,
    ConstParam,
    Constraint,
    Expr,
    ExprArray,
    ExprAssign,
    ExprAssignOp,
    ExprAsync,
    ExprAwait,
    ExprBinary,
    ExprBlock,
    ExprBox,
    ExprBreak,
    ExprCall,
    ExprCast,
    ExprClosure,
    ExprContinue,
    ExprField,
    ExprForLoop,
    ExprGroup,
    ExprIf,
    ExprIndex,
    ExprLet,
    ExprLit,
    ExprLoop,
    ExprMacro,
    ExprMatch,
    ExprMethodCall,
    ExprParen,
    ExprPath,
    ExprRange,
    ExprReference,
    ExprRepeat,
    ExprReturn,
    ExprStruct,
    ExprTry,
    ExprTryBlock,
    ExprTuple,
    ExprType,
    ExprUnary,
    ExprUnsafe,
    ExprWhile,
    ExprYield,
    Field,
    FieldPat,
    FieldValue,
    Fields,
    FieldsNamed,
    FieldsUnnamed,
    File,
    FnArg,
    ForeignItem,
    ForeignItemFn,
    ForeignItemMacro,
    ForeignItemStatic,
    ForeignItemType,
    GenericArgument,
    GenericMethodArgument,
    GenericParam,
    Generics,
    Ident,
    ImplItem,
    ImplItemConst,
    ImplItemMacro,
    ImplItemMethod,
    ImplItemType,
    Index,
    Item,
    ItemConst,
    ItemEnum,
    ItemExternCrate,
    ItemFn,
    ItemForeignMod,
    ItemImpl,
    ItemMacro,
    ItemMacro2,
    ItemMod,
    ItemStatic,
    ItemStruct,
    ItemTrait,
    ItemTraitAlias,
    ItemType,
    ItemUnion,
    ItemUse,
    Label,
    Lifetime,
    LifetimeDef,
    Lit,
    LitBool,
    LitByte,
    LitByteStr,
    LitChar,
    LitFloat,
    LitInt,
    LitStr,
    Local,
    Macro,
    Member,
    MethodTurbofish,
    ParenthesizedGenericArguments,
    Pat,
    PatBox,
    PatIdent,
    PatLit,
    PatMacro,
    PatOr,
    PatPath,
    PatRange,
    PatReference,
    PatRest,
    PatSlice,
    PatStruct,
    PatTuple,
    PatTupleStruct,
    PatType,
    PatWild,
    Path,
    PathArguments,
    PathSegment,
    PredicateEq,
    PredicateLifetime,
    PredicateType,
    QSelf,
    Receiver,
    ReturnType,
    Signature,
    Stmt,
    TraitBound,
    TraitBoundModifier,
    TraitItem,
    TraitItemConst,
    TraitItemMacro,
    TraitItemMethod,
    TraitItemType,
    Type,
    TypeArray,
    TypeBareFn,
    TypeGroup,
    TypeImplTrait,
    TypeInfer,
    TypeMacro,
    TypeNever,
    TypeParam,
    TypeParamBound,
    TypeParen,
    TypePath,
    TypePtr,
    TypeReference,
    TypeSlice,
    TypeTraitObject,
    TypeTuple,
    UnOp,
    UseGlob,
    UseGroup,
    UseName,
    UsePath,
    UseRename,
    UseTree,
    Variadic,
    Variant,
    VisCrate,
    VisPublic,
    VisRestricted,
    Visibility,
    WhereClause,
    WherePredicate,
    Comment,
}

impl<'a, 'b> From<&'a Syn<'b>> for SynKind {
    fn from(node: &'a Syn<'b>) -> Self {
        match node {
            Syn::Abi(..) => SynKind::Abi,
            Syn::AngleBracketedGenericArguments(..) => SynKind::AngleBracketedGenericArguments,
            Syn::Arm(..) => SynKind::Arm,
            Syn::AttrStyle(..) => SynKind::AttrStyle,
            Syn::Attribute(..) => SynKind::Attribute,
            Syn::BareFnArg(..) => SynKind::BareFnArg,
            Syn::BinOp(..) => SynKind::BinOp,
            Syn::Binding(..) => SynKind::Binding,
            Syn::Block(..) => SynKind::Block,
            Syn::BoundLifetimes(..) => SynKind::BoundLifetimes,
            Syn::ConstParam(..) => SynKind::ConstParam,
            Syn::Constraint(..) => SynKind::Constraint,
            Syn::Expr(..) => SynKind::Expr,
            Syn::ExprArray(..) => SynKind::ExprArray,
            Syn::ExprAssign(..) => SynKind::ExprAssign,
            Syn::ExprAssignOp(..) => SynKind::ExprAssignOp,
            Syn::ExprAsync(..) => SynKind::ExprAsync,
            Syn::ExprAwait(..) => SynKind::ExprAwait,
            Syn::ExprBinary(..) => SynKind::ExprBinary,
            Syn::ExprBlock(..) => SynKind::ExprBlock,
            Syn::ExprBox(..) => SynKind::ExprBox,
            Syn::ExprBreak(..) => SynKind::ExprBreak,
            Syn::ExprCall(..) => SynKind::ExprCall,
            Syn::ExprCast(..) => SynKind::ExprCast,
            Syn::ExprClosure(..) => SynKind::ExprClosure,
            Syn::ExprContinue(..) => SynKind::ExprContinue,
            Syn::ExprField(..) => SynKind::ExprField,
            Syn::ExprForLoop(..) => SynKind::ExprForLoop,
            Syn::ExprGroup(..) => SynKind::ExprGroup,
            Syn::ExprIf(..) => SynKind::ExprIf,
            Syn::ExprIndex(..) => SynKind::ExprIndex,
            Syn::ExprLet(..) => SynKind::ExprLet,
            Syn::ExprLit(..) => SynKind::ExprLit,
            Syn::ExprLoop(..) => SynKind::ExprLoop,
            Syn::ExprMacro(..) => SynKind::ExprMacro,
            Syn::ExprMatch(..) => SynKind::ExprMatch,
            Syn::ExprMethodCall(..) => SynKind::ExprMethodCall,
            Syn::ExprParen(..) => SynKind::ExprParen,
            Syn::ExprPath(..) => SynKind::ExprPath,
            Syn::ExprRange(..) => SynKind::ExprRange,
            Syn::ExprReference(..) => SynKind::ExprReference,
            Syn::ExprRepeat(..) => SynKind::ExprRepeat,
            Syn::ExprReturn(..) => SynKind::ExprReturn,
            Syn::ExprStruct(..) => SynKind::ExprStruct,
            Syn::ExprTry(..) => SynKind::ExprTry,
            Syn::ExprTryBlock(..) => SynKind::ExprTryBlock,
            Syn::ExprTuple(..) => SynKind::ExprTuple,
            Syn::ExprType(..) => SynKind::ExprType,
            Syn::ExprUnary(..) => SynKind::ExprUnary,
            Syn::ExprUnsafe(..) => SynKind::ExprUnsafe,
            Syn::ExprWhile(..) => SynKind::ExprWhile,
            Syn::ExprYield(..) => SynKind::ExprYield,
            Syn::Field(..) => SynKind::Field,
            Syn::FieldPat(..) => SynKind::FieldPat,
            Syn::FieldValue(..) => SynKind::FieldValue,
            Syn::Fields(..) => SynKind::Fields,
            Syn::FieldsNamed(..) => SynKind::FieldsNamed,
            Syn::FieldsUnnamed(..) => SynKind::FieldsUnnamed,
            Syn::File(..) => SynKind::File,
            Syn::FnArg(..) => SynKind::FnArg,
            Syn::ForeignItem(..) => SynKind::ForeignItem,
            Syn::ForeignItemFn(..) => SynKind::ForeignItemFn,
            Syn::ForeignItemMacro(..) => SynKind::ForeignItemMacro,
            Syn::ForeignItemStatic(..) => SynKind::ForeignItemStatic,
            Syn::ForeignItemType(..) => SynKind::ForeignItemType,
            Syn::GenericArgument(..) => SynKind::GenericArgument,
            Syn::GenericMethodArgument(..) => SynKind::GenericMethodArgument,
            Syn::GenericParam(..) => SynKind::GenericParam,
            Syn::Generics(..) => SynKind::Generics,
            Syn::Ident(..) => SynKind::Ident,
            Syn::ImplItem(..) => SynKind::ImplItem,
            Syn::ImplItemConst(..) => SynKind::ImplItemConst,
            Syn::ImplItemMacro(..) => SynKind::ImplItemMacro,
            Syn::ImplItemMethod(..) => SynKind::ImplItemMethod,
            Syn::ImplItemType(..) => SynKind::ImplItemType,
            Syn::Index(..) => SynKind::Index,
            Syn::Item(..) => SynKind::Item,
            Syn::ItemConst(..) => SynKind::ItemConst,
            Syn::ItemEnum(..) => SynKind::ItemEnum,
            Syn::ItemExternCrate(..) => SynKind::ItemExternCrate,
            Syn::ItemFn(..) => SynKind::ItemFn,
            Syn::ItemForeignMod(..) => SynKind::ItemForeignMod,
            Syn::ItemImpl(..) => SynKind::ItemImpl,
            Syn::ItemMacro(..) => SynKind::ItemMacro,
            Syn::ItemMacro2(..) => SynKind::ItemMacro2,
            Syn::ItemMod(..) => SynKind::ItemMod,
            Syn::ItemStatic(..) => SynKind::ItemStatic,
            Syn::ItemStruct(..) => SynKind::ItemStruct,
            Syn::ItemTrait(..) => SynKind::ItemTrait,
            Syn::ItemTraitAlias(..) => SynKind::ItemTraitAlias,
            Syn::ItemType(..) => SynKind::ItemType,
            Syn::ItemUnion(..) => SynKind::ItemUnion,
            Syn::ItemUse(..) => SynKind::ItemUse,
            Syn::Label(..) => SynKind::Label,
            Syn::Lifetime(..) => SynKind::Lifetime,
            Syn::LifetimeDef(..) => SynKind::LifetimeDef,
            Syn::Lit(..) => SynKind::Lit,
            Syn::LitBool(..) => SynKind::LitBool,
            Syn::LitByte(..) => SynKind::LitByte,
            Syn::LitByteStr(..) => SynKind::LitByteStr,
            Syn::LitChar(..) => SynKind::LitChar,
            Syn::LitFloat(..) => SynKind::LitFloat,
            Syn::LitInt(..) => SynKind::LitInt,
            Syn::LitStr(..) => SynKind::LitStr,
            Syn::Local(..) => SynKind::Local,
            Syn::Macro(..) => SynKind::Macro,
            Syn::Member(..) => SynKind::Member,
            Syn::MethodTurbofish(..) => SynKind::MethodTurbofish,
            Syn::ParenthesizedGenericArguments(..) => SynKind::ParenthesizedGenericArguments,
            Syn::Pat(..) => SynKind::Pat,
            Syn::PatBox(..) => SynKind::PatBox,
            Syn::PatIdent(..) => SynKind::PatIdent,
            Syn::PatLit(..) => SynKind::PatLit,
            Syn::PatMacro(..) => SynKind::PatMacro,
            Syn::PatOr(..) => SynKind::PatOr,
            Syn::PatPath(..) => SynKind::PatPath,
            Syn::PatRange(..) => SynKind::PatRange,
            Syn::PatReference(..) => SynKind::PatReference,
            Syn::PatRest(..) => SynKind::PatRest,
            Syn::PatSlice(..) => SynKind::PatSlice,
            Syn::PatStruct(..) => SynKind::PatStruct,
            Syn::PatTuple(..) => SynKind::PatTuple,
            Syn::PatTupleStruct(..) => SynKind::PatTupleStruct,
            Syn::PatType(..) => SynKind::PatType,
            Syn::PatWild(..) => SynKind::PatWild,
            Syn::Path(..) => SynKind::Path,
            Syn::PathArguments(..) => SynKind::PathArguments,
            Syn::PathSegment(..) => SynKind::PathSegment,
            Syn::PredicateEq(..) => SynKind::PredicateEq,
            Syn::PredicateLifetime(..) => SynKind::PredicateLifetime,
            Syn::PredicateType(..) => SynKind::PredicateType,
            Syn::QSelf(..) => SynKind::QSelf,
            Syn::Receiver(..) => SynKind::Receiver,
            Syn::ReturnType(..) => SynKind::ReturnType,
            Syn::Signature(..) => SynKind::Signature,
            Syn::Stmt(..) => SynKind::Stmt,
            Syn::TraitBound(..) => SynKind::TraitBound,
            Syn::TraitBoundModifier(..) => SynKind::TraitBoundModifier,
            Syn::TraitItem(..) => SynKind::TraitItem,
            Syn::TraitItemConst(..) => SynKind::TraitItemConst,
            Syn::TraitItemMacro(..) => SynKind::TraitItemMacro,
            Syn::TraitItemMethod(..) => SynKind::TraitItemMethod,
            Syn::TraitItemType(..) => SynKind::TraitItemType,
            Syn::Type(..) => SynKind::Type,
            Syn::TypeArray(..) => SynKind::TypeArray,
            Syn::TypeBareFn(..) => SynKind::TypeBareFn,
            Syn::TypeGroup(..) => SynKind::TypeGroup,
            Syn::TypeImplTrait(..) => SynKind::TypeImplTrait,
            Syn::TypeInfer(..) => SynKind::TypeInfer,
            Syn::TypeMacro(..) => SynKind::TypeMacro,
            Syn::TypeNever(..) => SynKind::TypeNever,
            Syn::TypeParam(..) => SynKind::TypeParam,
            Syn::TypeParamBound(..) => SynKind::TypeParamBound,
            Syn::TypeParen(..) => SynKind::TypeParen,
            Syn::TypePath(..) => SynKind::TypePath,
            Syn::TypePtr(..) => SynKind::TypePtr,
            Syn::TypeReference(..) => SynKind::TypeReference,
            Syn::TypeSlice(..) => SynKind::TypeSlice,
            Syn::TypeTraitObject(..) => SynKind::TypeTraitObject,
            Syn::TypeTuple(..) => SynKind::TypeTuple,
            Syn::UnOp(..) => SynKind::UnOp,
            Syn::UseGlob(..) => SynKind::UseGlob,
            Syn::UseGroup(..) => SynKind::UseGroup,
            Syn::UseName(..) => SynKind::UseName,
            Syn::UsePath(..) => SynKind::UsePath,
            Syn::UseRename(..) => SynKind::UseRename,
            Syn::UseTree(..) => SynKind::UseTree,
            Syn::Variadic(..) => SynKind::Variadic,
            Syn::Variant(..) => SynKind::Variant,
            Syn::VisCrate(..) => SynKind::VisCrate,
            Syn::VisPublic(..) => SynKind::VisPublic,
            Syn::VisRestricted(..) => SynKind::VisRestricted,
            Syn::Visibility(..) => SynKind::Visibility,
            Syn::WhereClause(..) => SynKind::WhereClause,
            Syn::WherePredicate(..) => SynKind::WherePredicate,
            Syn::Comment(..) => SynKind::Comment,
        }
    }
}

impl<'a> Syn<'a> {
    pub fn attributes(&self) -> Option<&[syn::Attribute]> {
        use Syn::*;
        match self {
            Arm(node) => Some(&node.attrs[..]),
            BareFnArg(node) => Some(&node.attrs[..]),
            ConstParam(node) => Some(&node.attrs[..]),
            ExprArray(node) => Some(&node.attrs[..]),
            ExprAssign(node) => Some(&node.attrs[..]),
            ExprAssignOp(node) => Some(&node.attrs[..]),
            ExprAsync(node) => Some(&node.attrs[..]),
            ExprAwait(node) => Some(&node.attrs[..]),
            ExprBinary(node) => Some(&node.attrs[..]),
            ExprBlock(node) => Some(&node.attrs[..]),
            ExprBox(node) => Some(&node.attrs[..]),
            ExprBreak(node) => Some(&node.attrs[..]),
            ExprCall(node) => Some(&node.attrs[..]),
            ExprCast(node) => Some(&node.attrs[..]),
            ExprClosure(node) => Some(&node.attrs[..]),
            ExprContinue(node) => Some(&node.attrs[..]),
            ExprField(node) => Some(&node.attrs[..]),
            ExprForLoop(node) => Some(&node.attrs[..]),
            ExprGroup(node) => Some(&node.attrs[..]),
            ExprIf(node) => Some(&node.attrs[..]),
            ExprIndex(node) => Some(&node.attrs[..]),
            ExprLet(node) => Some(&node.attrs[..]),
            ExprLit(node) => Some(&node.attrs[..]),
            ExprLoop(node) => Some(&node.attrs[..]),
            ExprMacro(node) => Some(&node.attrs[..]),
            ExprMatch(node) => Some(&node.attrs[..]),
            ExprMethodCall(node) => Some(&node.attrs[..]),
            ExprParen(node) => Some(&node.attrs[..]),
            ExprPath(node) => Some(&node.attrs[..]),
            ExprRange(node) => Some(&node.attrs[..]),
            ExprReference(node) => Some(&node.attrs[..]),
            ExprRepeat(node) => Some(&node.attrs[..]),
            ExprReturn(node) => Some(&node.attrs[..]),
            ExprStruct(node) => Some(&node.attrs[..]),
            ExprTry(node) => Some(&node.attrs[..]),
            ExprTryBlock(node) => Some(&node.attrs[..]),
            ExprTuple(node) => Some(&node.attrs[..]),
            ExprType(node) => Some(&node.attrs[..]),
            ExprUnary(node) => Some(&node.attrs[..]),
            ExprUnsafe(node) => Some(&node.attrs[..]),
            ExprWhile(node) => Some(&node.attrs[..]),
            ExprYield(node) => Some(&node.attrs[..]),
            Field(node) => Some(&node.attrs[..]),
            FieldPat(node) => Some(&node.attrs[..]),
            FieldValue(node) => Some(&node.attrs[..]),
            File(node) => Some(&node.attrs[..]),
            ForeignItemFn(node) => Some(&node.attrs[..]),
            ForeignItemMacro(node) => Some(&node.attrs[..]),
            ForeignItemStatic(node) => Some(&node.attrs[..]),
            ForeignItemType(node) => Some(&node.attrs[..]),
            ImplItemConst(node) => Some(&node.attrs[..]),
            ImplItemMacro(node) => Some(&node.attrs[..]),
            ImplItemMethod(node) => Some(&node.attrs[..]),
            ImplItemType(node) => Some(&node.attrs[..]),
            ItemConst(node) => Some(&node.attrs[..]),
            ItemEnum(node) => Some(&node.attrs[..]),
            ItemExternCrate(node) => Some(&node.attrs[..]),
            ItemFn(node) => Some(&node.attrs[..]),
            ItemForeignMod(node) => Some(&node.attrs[..]),
            ItemImpl(node) => Some(&node.attrs[..]),
            ItemMacro(node) => Some(&node.attrs[..]),
            ItemMacro2(node) => Some(&node.attrs[..]),
            ItemMod(node) => Some(&node.attrs[..]),
            ItemStatic(node) => Some(&node.attrs[..]),
            ItemStruct(node) => Some(&node.attrs[..]),
            ItemTrait(node) => Some(&node.attrs[..]),
            ItemTraitAlias(node) => Some(&node.attrs[..]),
            ItemType(node) => Some(&node.attrs[..]),
            ItemUnion(node) => Some(&node.attrs[..]),
            ItemUse(node) => Some(&node.attrs[..]),
            LifetimeDef(node) => Some(&node.attrs[..]),
            Local(node) => Some(&node.attrs[..]),
            PatBox(node) => Some(&node.attrs[..]),
            PatIdent(node) => Some(&node.attrs[..]),
            PatLit(node) => Some(&node.attrs[..]),
            PatMacro(node) => Some(&node.attrs[..]),
            PatOr(node) => Some(&node.attrs[..]),
            PatPath(node) => Some(&node.attrs[..]),
            PatRange(node) => Some(&node.attrs[..]),
            PatReference(node) => Some(&node.attrs[..]),
            PatSlice(node) => Some(&node.attrs[..]),
            PatStruct(node) => Some(&node.attrs[..]),
            PatTuple(node) => Some(&node.attrs[..]),
            PatTupleStruct(node) => Some(&node.attrs[..]),
            PatType(node) => Some(&node.attrs[..]),
            PatWild(node) => Some(&node.attrs[..]),
            Receiver(node) => Some(&node.attrs[..]),
            TraitItemConst(node) => Some(&node.attrs[..]),
            TraitItemMacro(node) => Some(&node.attrs[..]),
            TraitItemMethod(node) => Some(&node.attrs[..]),
            TraitItemType(node) => Some(&node.attrs[..]),
            TypeParam(node) => Some(&node.attrs[..]),
            Variadic(node) => Some(&node.attrs[..]),
            Variant(node) => Some(&node.attrs[..]),
            _ => None,
        }
    }
}
macro_rules! derive_from {
    ($variant:ident) => {
        impl<'a> From<&'a syn::$variant> for Syn<'a> {
            fn from(i: &'a syn::$variant) -> Self {
                Syn::$variant(i)
            }
        }

        impl<'a> From<&'a syn::$variant> for SynKind {
            fn from(_: &'a syn::$variant) -> Self {
                SynKind::$variant
            }
        }
    };
}

derive_from![Abi];
derive_from![AngleBracketedGenericArguments];
derive_from![Arm];
derive_from![AttrStyle];
derive_from![Attribute];
derive_from![BareFnArg];
derive_from![BinOp];
derive_from![Binding];
derive_from![Block];
derive_from![BoundLifetimes];
derive_from![ConstParam];
derive_from![Constraint];
derive_from![Expr];
derive_from![ExprArray];
derive_from![ExprAssign];
derive_from![ExprAssignOp];
derive_from![ExprAsync];
derive_from![ExprAwait];
derive_from![ExprBinary];
derive_from![ExprBlock];
derive_from![ExprBox];
derive_from![ExprBreak];
derive_from![ExprCall];
derive_from![ExprCast];
derive_from![ExprClosure];
derive_from![ExprContinue];
derive_from![ExprField];
derive_from![ExprForLoop];
derive_from![ExprGroup];
derive_from![ExprIf];
derive_from![ExprIndex];
derive_from![ExprLet];
derive_from![ExprLit];
derive_from![ExprLoop];
derive_from![ExprMacro];
derive_from![ExprMatch];
derive_from![ExprMethodCall];
derive_from![ExprParen];
derive_from![ExprPath];
derive_from![ExprRange];
derive_from![ExprReference];
derive_from![ExprRepeat];
derive_from![ExprReturn];
derive_from![ExprStruct];
derive_from![ExprTry];
derive_from![ExprTryBlock];
derive_from![ExprTuple];
derive_from![ExprType];
derive_from![ExprUnary];
derive_from![ExprUnsafe];
derive_from![ExprWhile];
derive_from![ExprYield];
derive_from![Field];
derive_from![FieldPat];
derive_from![FieldValue];
derive_from![Fields];
derive_from![FieldsNamed];
derive_from![FieldsUnnamed];
derive_from![File];
derive_from![FnArg];
derive_from![ForeignItem];
derive_from![ForeignItemFn];
derive_from![ForeignItemMacro];
derive_from![ForeignItemStatic];
derive_from![ForeignItemType];
derive_from![GenericArgument];
derive_from![GenericMethodArgument];
derive_from![GenericParam];
derive_from![Generics];
impl<'a> From<&'a proc_macro2::Ident> for Syn<'a> {
    fn from(ident: &'a proc_macro2::Ident) -> Self {
        Syn::Ident(ident)
    }
}
derive_from![ImplItem];
derive_from![ImplItemConst];
derive_from![ImplItemMacro];
derive_from![ImplItemMethod];
derive_from![ImplItemType];
derive_from![Index];
derive_from![Item];
derive_from![ItemConst];
derive_from![ItemEnum];
derive_from![ItemExternCrate];
derive_from![ItemFn];
derive_from![ItemForeignMod];
derive_from![ItemImpl];
derive_from![ItemMacro];
derive_from![ItemMacro2];
derive_from![ItemMod];
derive_from![ItemStatic];
derive_from![ItemStruct];
derive_from![ItemTrait];
derive_from![ItemTraitAlias];
derive_from![ItemType];
derive_from![ItemUnion];
derive_from![ItemUse];
derive_from![Label];
derive_from![Lifetime];
derive_from![LifetimeDef];
derive_from![Lit];
derive_from![LitBool];
derive_from![LitByte];
derive_from![LitByteStr];
derive_from![LitChar];
derive_from![LitFloat];
derive_from![LitInt];
derive_from![LitStr];
derive_from![Local];
derive_from![Macro];
derive_from![Member];
derive_from![MethodTurbofish];
derive_from![ParenthesizedGenericArguments];
derive_from![Pat];
derive_from![PatBox];
derive_from![PatIdent];
derive_from![PatLit];
derive_from![PatMacro];
derive_from![PatOr];
derive_from![PatPath];
derive_from![PatRange];
derive_from![PatReference];
derive_from![PatRest];
derive_from![PatSlice];
derive_from![PatStruct];
derive_from![PatTuple];
derive_from![PatTupleStruct];
derive_from![PatType];
derive_from![PatWild];
derive_from![Path];
derive_from![PathArguments];
derive_from![PathSegment];
derive_from![PredicateEq];
derive_from![PredicateLifetime];
derive_from![PredicateType];
derive_from![QSelf];
derive_from![Receiver];
derive_from![ReturnType];
derive_from![Signature];
derive_from![Stmt];
derive_from![TraitBound];
derive_from![TraitBoundModifier];
derive_from![TraitItem];
derive_from![TraitItemConst];
derive_from![TraitItemMacro];
derive_from![TraitItemMethod];
derive_from![TraitItemType];
derive_from![Type];
derive_from![TypeArray];
derive_from![TypeBareFn];
derive_from![TypeGroup];
derive_from![TypeImplTrait];
derive_from![TypeInfer];
derive_from![TypeMacro];
derive_from![TypeNever];
derive_from![TypeParam];
derive_from![TypeParamBound];
derive_from![TypeParen];
derive_from![TypePath];
derive_from![TypePtr];
derive_from![TypeReference];
derive_from![TypeSlice];
derive_from![TypeTraitObject];
derive_from![TypeTuple];
derive_from![UnOp];
derive_from![UseGlob];
derive_from![UseGroup];
derive_from![UseName];
derive_from![UsePath];
derive_from![UseRename];
derive_from![UseTree];
derive_from![Variadic];
derive_from![Variant];
derive_from![VisCrate];
derive_from![VisPublic];
derive_from![VisRestricted];
derive_from![Visibility];
derive_from![WhereClause];
derive_from![WherePredicate];
