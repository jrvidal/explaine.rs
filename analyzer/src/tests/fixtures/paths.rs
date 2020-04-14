span: [1, 0] => [1, 4]
item: ReceiverPath
---
self<|>.foo();

---
span: [3, 4] => [3, 8]
item: ReceiverPath method: bark
---
impl Foobar {
  fn bark(&self) {
    self<|>.foo();
  }
}

---
span: [3, 4] => [3, 8]
item: ReceiverPath method: bark
---
trait Foobar {
  fn bark(&self) {
    self<|>.foo();
  }
}

---
span: [5, 8] => [5, 12]
item: ReceiverPath method: growl
---
impl Foobar {
  fn bark(&self) {
    impl Baz {
      fn growl(&mut self) {
        self<|>.foo();
      }
    }
  }
}
