span: [1, 0] => [1, 10]
item: ItemStruct name: Foo, unit: false, generics: { lifetime: false, type_: true, const_: false }
---
struct <|>Foo<T> {
  x: T
}
---
span: [1, 0] => [1, 10]
item: LifetimeParam name: "a", of: Struct, of_name: Foo
---
struct Foo<'a<|>> { }
