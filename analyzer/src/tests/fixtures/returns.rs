span: [2, 2] => [2, 8]
item: ExprReturn of: Function
naked: true
---
fn main() {
  ret<|>urn;
}
---
span: [3, 4] => [3, 10]
item: ExprReturn of: Method
naked: true
---
trait Foo {
  fn main() {
    ret<|>urn;
  }
}
---
span: [3, 4] => [3, 10]
item: ExprReturn of: Method
naked: true
---
impl Foo {
  fn main() {
    ret<|>urn;
  }
}
---
span: [2, 2] => [2, 8]
item: ExprReturn of: Closure
---
|| {
  ret<|>urn;
}
---
span: [2, 2] => [2, 8]
item: ExprReturn of: AsyncBlock
---
async {
  ret<|>urn;
}