span: [1, 0] => [1, 9]
item: Comment block: true
naked: true
---
/*<|> foo */
fn foo() {}

---
span: [3, 0] => [3, 6]
item: Comment block: false
naked: true
---
fn foo() {}
// foo
// <|>bar
---
span: [1, 1] => [1, 8]
item: Comment block: true
---
(/*foo*/<|>x)
