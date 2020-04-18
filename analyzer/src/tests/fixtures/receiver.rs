span: [1, 7] => [1, 11]
item: ValueSelf mutability: false, explicit: false
---
fn foo(<|>self) {}

---
span: [1, 7] => [1, 15]
item: ValueSelf mutability: true, explicit: false
---
fn foo(mut <|>self) {}

---
span: [1, 7] => [1, 16]
item: MutSelf explicit: false, mutability: false
---
fn foo(<|>&mut self) {}

---
span: [1, 7] => [1, 12]
item: RefSelf explicit: false, mutability: false
---
fn foo(<|>&self) {}

---
span: [1, 7] => [1, 17]
item: ValueSelf explicit: true, mutability: false
---
fn foo(self:<|> Self) {}

---
span: [1, 7] => [1, 18]
item: RefSelf explicit: true, mutability: false
---
fn foo(self:<|> &Self) {}

---
span: [1, 7] => [1, 25]
item: MutSelf explicit: true, mutability: false
---
fn foo(self:<|> &'a mut Self) {}
