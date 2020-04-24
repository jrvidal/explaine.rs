span: [1, 0] => [1, 4]
item: Macro
---
foo!<|>[bar baz];

---
span: [1, 5] => [1, 13]
item: MacroTokens
---
foo![bar <|> baz];

---
span: [1, 0] => [2, 1]
item: ItemMacroRules name: foo
naked: true
---
macro_rules! foo<|> {
}

---
span: [1, 0] => [2, 1]
item: ItemMacroRules name: foo
naked: true
---
macro_rules! foo {<|>
}
