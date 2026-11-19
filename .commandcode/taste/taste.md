# Taste

## Coding style

- Pushes for idiomatic, DRY Rust: asks "is this the most idiomatic way?" and "any DRYier way of doing this?" — prefer shared helpers/abstractions (e.g. an operator table) over duplicated near-identical code. Confidence: 0.6
- Declines extra indirection invented just to please the compiler (helper functions, type aliases/annotations) — responds "ehhh, I'll just use it" and prefers wiring the code up so inference resolves it. Confidence: 0.55

## Workflow

- Keep fixes tightly scoped: fix only the reported bug/compile error, and don't slip in unrelated changes (e.g. whitespace/padding behavior) — leave those for him to decide. Confidence: 0.85

## Communication

- Wants thorough, line-by-line explanations of the code he's given (he's on a learning project and says "I want to understand as many lines of code as possible to learn parsing and languages") rather than just a working snippet; phrases requests as "teach me how to fix this" — explain the cause and options, don't just apply an edit. Confidence: 0.85
