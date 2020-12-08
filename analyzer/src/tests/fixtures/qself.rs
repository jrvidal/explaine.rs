span: [1, 0] => [1, 3]
item: QSelf as_trait: false
---
<F><|>::foo

---
span: [1, 0] => [1, 10]
item: QSelf as_trait: true
---
<F as<|> Bar>::foo

---
span: [1, 0] => [1, 10]
item: QSelf as_trait: true
---
<F<|> as Bar>::foo

---
span: [1, 10] => [1, 17]
item: StaticLifetime
---
<F as Bar<<|>'static>>::foo
