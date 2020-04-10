span: [1, 7] => [1, 11]
item: TypeBareFn
---
let x: fn(<|>);

---
span: [1, 12] => [1, 14]
item: RArrow return_of: BareFunctionType
---
let x: fn() <|>-> ();

---
span: [1, 7] => [1, 14]
item: BoundLifetimesBareFnType
---
let x: fo<|>r<'a> fn();