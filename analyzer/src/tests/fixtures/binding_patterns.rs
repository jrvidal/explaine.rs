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
