span: [1, 4] => [1, 13]
item: PatStruct empty: false, bindings: Let
---
let Foo<|> { x } = y;

---
span: [1, 7] => [1, 13]
item: PatTuple single_comma: false, bindings: Arg
---
fn foo((x, y)<|>: Type) {}

---
span: [1, 4] => [1, 10]
item: PatTuple single_comma: false, bindings: ForLoop
---
for (x, y)<|> in x {}

---
span: [1, 4] => [1, 5]
item: ForLoopLocal mutability: false, ident: x
---
for x<|> in x {}

---
span: [1, 4] => [1, 9]
item: ForLoopLocal mutability: true, ident: x
---
for mut x<|> in x {}

---
span: [1, 4] => [1, 9]
item: ForLoopLocal mutability: true, ident: x
---
for <|>mut x in x {}
