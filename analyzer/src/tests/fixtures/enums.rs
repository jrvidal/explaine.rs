span: [4, 6] => [4, 7]
item: Variant name: Foo
---
enum Bar {
  A = {
    enum Foo {
      A<|>
    }

    1
  }
}
