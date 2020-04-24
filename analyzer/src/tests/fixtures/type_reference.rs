span: [1, 7] => [1, 9]
item: TypeReference mutable: false, lifetime: false, ty: T
---
let x: &<|>T;

---
span: [1, 7] => [1, 11]
item: KnownTypeStrSlice mutability: false
---
let x: &str<|>;

---
span: [1, 7] => [1, 12]
item: TypeSlice dynamic: true, ty: u8
---
let x: &[u8]<|>;
