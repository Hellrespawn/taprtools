# tfmttools

## Modes of operations

### Clear History

Requires:

- Preview
- History File Location
- History

### List Scripts

- Script File Location

### Inspect Script

- Script File Location
- Name of script

### Undo/Redo

- Preview
- History File Location
- History
- \# steps to undo

### Rename files

- Preview
- Script File Location
- History File Location
- History
- Name of script
- Arguments of script

## Operators

Operators in order of precedence, from lowest to highest:

| Operator                             | Notes                                         |
|:------------------------------------:|:---------------------------------------------:|
| `a ? b : c`                          | `if a then b else c`                          |
| `a \| b`<br>`a \|\| b`               | `if a then a else b`<br>`if a then ab else b` |
| `a & b`<br>`a && b`                  | `if a then b else a`<br>`if a then ab else a` |
| `a + b`<br>`a - b`                   |                                               |
| `a * b`<br>`a / b`<br>`a % b`        | `/` does integer division only.               |
| `a ** b`<br>`a ^ b`                  | Exponentiation                                |
| `+ a`<br>`- a`<br>`(expressions...)` |                                               |

## To-do

- TODO Better UX
- TODO? Add more obscure tags?
- TODO? Add strict mode, which allows/denies/errors on forbidden characters/directory separators.
