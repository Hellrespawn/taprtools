# tfmttools

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
