span: [1, 7] => [1, 11]
item: ValueSelf mutability: false
---
fn foo(<|>self) {}

---
span: [1, 7] => [1, 15]
item: ValueSelf mutability: true
---
fn foo(mut <|>self) {}

---
span: [1, 7] => [1, 16]
item: MutSelf
---
fn foo(<|>&mut self) {}

---
span: [1, 7] => [1, 12]
item: RefSelf
---
fn foo(<|>&self) {}
