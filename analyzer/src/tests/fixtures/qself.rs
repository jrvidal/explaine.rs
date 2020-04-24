span: [1, 0] => [1, 3]
item: QSelf
---
<F><|>::foo

---
span: [1, 0] => [1, 10]
item: QSelfAsTrait
---
<F as<|> Bar>::foo

---
span: [1, 0] => [1, 10]
item: QSelfAsTrait
---
<F<|> as Bar>::foo

---
span: [1, 10] => [1, 17]
item: StaticLifetime
---
<F as Bar<<|>'static>>::foo
