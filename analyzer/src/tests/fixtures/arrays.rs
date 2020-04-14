span: [1, 4] => [1, 10]
item: ExprArray
---
x = <|>[1, 2];

---
span: [1, 4] => [1, 11]
item: ExprArraySlice
---
x = &[1, 2]<|>;

---
span: [1, 4] => [1, 11]
item: ExprArraySlice
---
x = <|>&[1, 2];