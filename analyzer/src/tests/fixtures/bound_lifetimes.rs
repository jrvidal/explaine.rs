span: [1, 14] => [1, 21]
item: BoundLifetimesTraitBound multiple: false, lifetime: "'a", ty: B
naked: true
---
type A = &dyn for<'a<|>> B;

---
span: [1, 9] => [1, 16]
item: BoundLifetimesBareFnType
naked: true
---
type A = for<'a<|>> fn(&'a u32);

---
span: [1, 18] => [1, 25]
item: BoundLifetimes
naked: true
---
fn foo<T>() where for<|><'a> T: B<'a> {}
