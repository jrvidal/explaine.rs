span: [1, 0] => [1, 9]
item: Local mutability: true, ident: x
---
let <|>mut x;

---
span: [1, 0] => [1, 9]
item: Local mutability: true, ident: x
---
let <|>mut x: i32;

---
span: [1, 0] => [1, 5]
item: Local mutability: false, ident: x
---
let <|>x: i32;

---
span: [1, 4] => [1, 10]
item: PatTuple bindings: Let
---
let <|>(x, y): (A, B);