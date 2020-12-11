span: [1, 4] => [1, 6]
item: PathLeadingColon
---
use <|>::foo;

---
span: [1, 4] => [1, 9]
item: PathSegmentCrate
---
use crat<|>e::foo;

---
span: [1, 15] => [1, 20]
item: PathSegmentCrate
---
use {foo::bar, crat<|>e::foo};

---
span: [1, 16] => [1, 21]
item: PathSegmentSuper
---
use {{foo::bar, <|>super::bar}, crate::foo};

---
span: [1, 10] => [1, 14]
item: UseGroupSelf parent: foo
---
use foo::{<|>self};

---
span: [1, 10] => [1, 15]
item: PathSegmentSuper
---
use self::su<|>per::foo;

---
span: [1, 8] => [1, 14]
item: AsRename underscore: false
---
use foo as <|>bar;
---
span: [1, 8] => [1, 12]
item: AsRename underscore: true
---
use foo as<|> _;
