# rw

rw is a simple language using an rewriting engine and unification for term evaluation.
The language lets you define rewriting rules

```text
let a => c
```

for replacing a pattern `a` with a template `c`. The `a => b` mapping is unidirectional,
for bidirectional mapping use `a <=> b`. More advanced patterns are also possible:

* `_` is a wildcard pattern that matches anything.
* `~0` would match non-zeros. `~` would negate any pattern following it.
* `$A` is a pattern variable, it matches anything and then every occurrence
  of the same pattern variable in the template is replaced with the matched value.
  Pattern variable names always start with `$`.
* Pattern variables can be match using patterns, for example `$A:(_ 0)`
  would match `$A` only to the list where the second element is zero. The patterns
  are attached to the variables using `:`. They can also consist of other
  variables, for example `$A:($B 0)` would be similar to the previous pattern, but
  would save the head of the list as `$B`.

Syntactic sugar can be used for defining functions mapping different inputs
to outputs, for example:

```text
let nonzero {
  0 => false
  ~0 => true
}

rewrite nonzero 5
rewrite nonzero 0
```

The rewriting rules can be evaluated using `rewrite`. The rules are applied
to the given query until no more simplifications are possible. The result
of the simplification is printed.

```text
infix != 10
let $A != $B => not ($A = $B)

rewrite true != false
```

As shown in the example above, user can define their own operators using
`infix <name> <precedence>`. The actions assigned to operators can be defined
using rewriting rules. Rewriting rules can serve as function definitions.

The language supports the following data types:

* literals like `a`, `Foo`, `42`, `true`, or `"hello, world!"`
* lists `a b c` which technically are cons pairs, as in lisp `(a (b c))`

Lists can be used to imitate behavior of functions, prefix operators, etc.

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
