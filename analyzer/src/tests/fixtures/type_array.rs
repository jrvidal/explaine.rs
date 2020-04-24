span: [1, 7] => [1, 15]
item: TypeArray
---
let x: <|>[u32; 3];

---
span: [1, 7] => [1, 15]
item: TypeArray
---
let x: [u32; <|>3];

---
span: [1, 7] => [1, 15]
item: TypeArray
---
let x: [u32; <|>N];

---
span: [1, 8] => [1, 11]
item: KnownTypeU32
---
let x: [u32<|>; 3];
