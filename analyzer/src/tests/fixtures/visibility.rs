span: [1, 0] => [1, 3]
item: VisPublic
---
pub<|> type A = B;

---
span: [1, 0] => [1, 5]
item: VisCrate
---
crate<|> type A = B;

---
span: [1, 0] => [1, 10]
item: VisRestricted path: Super, in_: false
---
pub(<|>super) type A = B;

---
span: [1, 0] => [1, 9]
item: VisRestricted path: Self_, in_: false
---
pub(<|>self) type A = B;

---
span: [1, 0] => [1, 10]
item: VisRestricted path: Crate, in_: false
---
pub(<|>crate) type A = B;

---
span: [1, 0] => [1, 13]
item: VisRestricted path: Crate, in_: true
---
pub(in <|>crate) type A = B;
