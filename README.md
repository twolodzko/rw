# rw

rw is a simple language using an rewriting engine and unification for term evaluation.
The language lets you define facts such as

```test
let a
let not b
```

for `a` being true and `b` being false. You can also define rewriting rules

```text
let a => c
let $A + 0 => $A
```

for replacing every `a` with `c` and a templated rule replacing occurrences of
`<something> + 0` with `<something>`. The `a => b` mapping is unidirectional,
for bidirectional mapping use `a = b`. Variable names in rewriting rules 
start with "$", "_" is a wildcard that can replace anything else in the pattern.

Syntactic sugar can be used for defining functions mapping different inputs
to outputs, for example:

```text
let nonzero {
  0 => false
  $A => true
}

eval nonzero 5
eval nonzero 0
```

The rewriting rules can be evaluated using `eval`:

```text
infix != 10
let $A != $B => not ($A = $B)

eval true != false
```

The evaluation simplifies the given query until no more simplifications
are possible to the query evaluates directly to a boolean value. The result
of the evaluation is printed.

As shown in the example above, user can define their own operators using
`prefix <name> <precedence>` and `infix <name> <precedence>`. The actions
assigned to operators can be defined using rewriting rules. Rewriting
rules can serve as function definitions.

The language supports the following data types:

* `true` and `false` booleans
* literals like `a`, `Foo`, `42`, or `"hello, world!"`
* lists `a b c` which technically are cons pairs, as in lisp `(a (b c))`

## TODO

```text
let foo :
| A B => C
| D E => F

```

* `let key := expr` --> `var &key := expr`, and `&key`. The variables are not mappings, they are
  directly replaced in the statements during the evaluation. They are expanded during "let" step
  and not allowed during dynamic dispatch.
* semicolon at end of line

```text
open "path/to/file" as name

class algebra {
  let ...
  let x := ...
  eval ...
  assert ...
  ...
}

eval 2 + 2 = 4 by algebra logic

```

## Sets

```text
var S := {}
let x in S
let y in S

eval all x in S: not x
```

## Functions

* Function is not a rule, it needs an explicit call.
* If function does not match the arguments it fails with an error.
* Function has it's own context.

```text
fun foo A B ... => ...

fun foo A B ... => {
  ...
  ...
}

fun foo
   | A B ... => ...
   | _ => ...

# function for a simple rewrite
fun add_neg X + -Y => X - Y
```

## Resources

* https://en.wikipedia.org/wiki/Rewriting
* https://stackoverflow.com/questions/7540227/strategies-for-simplifying-math-expressions
* https://www.cs.tau.ac.il/~nachum/papers/klop.pdf
* https://williamr.dev/posts/rw-systems/
* https://www.cs.tau.ac.il/~nachum/papers/hand-final.pdf
* https://ncatlab.org/nlab/show/rewriting
* https://web.archive.org/web/20241113163043/https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=1277313dd2c4fce640780b7472221156579e392f
* http://infolab.stanford.edu/pub/cstr/reports/cs/tr/80/785/CS-TR-80-785.pdf
* https://www.cs.tau.ac.il/~nachum/papers/taste-fixed.pdf
* https://homepages.cwi.nl/~daybuild/daily-books/extraction-transformation/term-rewriting/term-rewriting.html
* https://jf-parent.github.io/blog/2026/03/08/rewriting_system/
* https://en.wikipedia.org/wiki/Knuth%E2%80%93Bendix_completion_algorithm
