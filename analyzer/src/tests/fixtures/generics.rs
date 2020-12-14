span: [1, 0] => [1, 10]
item: ItemStruct name: Foo, unit: false, generic: true
---
struct <|>Foo<T> {
  x: T
}

---
span: [1, 11] => [1, 13]
item: LifetimeParam name: "a", of: Struct, of_name: Foo
---
struct Foo<'a<|>> { }

---
span: [1, 21] => [1, 25]
item: TypeParamUse of: Struct, of_name: Foo, implementation: false, name: TYPE
---
struct Foo<TYPE>{ x: TYPE<|> }

---
span: [3, 12] => [3, 13]
item: TypeParamUse of: ImplMethod, of_name: foo, implementation: false, name: B
---
impl<A> Foo<A>{
  fn foo<B>() {
      bar::<B<|>>();
  }
}

---
span: [3, 12] => [3, 13]
item: TypeParamUse of: Impl, of_name: "", implementation: true, name: A
---
impl<A> Foo<A>{
  fn foo() {
      bar::<A<|>>();
  }
}
---
span: [2, 13] => [2, 14]
item: TypeParamUse of: Fn, of_name: foo, implementation: false, name: A
---
fn foo<A>() {
  let x: Baz<A<|>>;
}
---
span: [0, 0] => [0, 0]
item: null
---
fn foo<A>() {
  fn bar() {
    let x: Baz<A<|>>;
  }
}
---
span: [1, 7] => [1, 8]
item: TypeParam name: A, of: Fn, of_name: foo
---
fn foo<A<|>>() {}
---
span: [1, 24] => [1, 26]
item: LifetimeParamUse name: a, of: BoundLifetime, of_name: "", implementation: false
---
fn foo(fun: for<'a> fn(&'<|>a u32)) {}
