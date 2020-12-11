span: [1, 0] => [1, 12]
item: ItemExternCrate
---
extern<|> crate foo;

---
span: [1, 17] => [1, 23]
item: AsRenameExternCrate underscore: false
---
extern crate foo as<|> bar;

---
span: [1, 17] => [1, 21]
item: AsRenameExternCrate underscore: true
---
extern crate foo as _<|>;
