span: [1, 0] => [1, 3]
item: Label loop_of: Loop
---
'a<|>: loop {}

---
span: [1, 0] => [1, 3]
item: Label loop_of: For
---
'a<|>: for x in y {}

---
span: [1, 0] => [1, 3]
item: Label loop_of: WhileLet
---
'a:<|> while let Some(x) = y {}

---
span: [1, 0] => [1, 3]
item: Label loop_of: While
---
<|>'a: while true {}

---
span: [1, 7] => [1, 12]
item: ExprBreak expr: false
---
loop { <|>break; }

---
span: [1, 7] => [1, 15]
item: ExprBreak expr: false, label: "'a"
---
loop { break<|> 'a; }

---
span: [1, 7] => [1, 14]
item: ExprBreak expr: true
---
loop { break<|> 7; }

---
span: [1, 7] => [1, 17]
item: ExprBreak expr: true, label: "'c"
---
loop { break <|>'c 7; }

---
span: [1, 16] => [1, 17]
item: LitInt separators: false
---
loop { break 'c 7<|>; }
