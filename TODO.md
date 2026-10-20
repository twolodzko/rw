# TODO

* OR patterns: `a|b|c => x`, merge other patterns to the OR patterns if feasible
* Commutativity can be combined with OR patterns (?)
* NOT patterns `~a => x` can lead to rules always match like `let ~0 => true` but would often be skipped
  because it is possible to apply same rule again so there is a cycle
* Above probably needs also `~0 as $A => $A` or Rust's `$A @ ~a => $A`, or `$A:~0` or `$A if ~0`

   ```text
   let $A:(_ $B) => ($A $B)  # (a b) => (a b b ...)
   let $A:(a|b|c) => ok
   ```

   `<var>:<atom>` where `$A:_` is the same as `$A`, or `($A $B:$A)` is the same as `($A $A)` but with additional variable name created
