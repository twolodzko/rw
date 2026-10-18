#!/usr/bin/env bats

BIN='./rw'

@test "examples/cycle.rw" {
   run $BIN examples/cycle.rw
   [ "$status" -eq 0 ]
}

@test "examples/algebra.rw" {
   run $BIN examples/algebra.rw
   [ "$status" -eq 0 ]
}

@test "examples/peano.rw" {
   run $BIN examples/peano.rw
   [ "$status" -eq 0 ]
}

@test "examples/lists.rw" {
   run $BIN examples/lists.rw
   [ "$status" -eq 0 ]
}

@test "examples/rules.rw" {
   run $BIN examples/rules.rw
   [ "$status" -eq 0 ]
}

@test "examples/variables.rw" {
   run $BIN examples/variables.rw
   [ "$status" -eq 0 ]
}

@test "examples/operators.rw" {
   run $BIN examples/operators.rw
   [ "$status" -eq 0 ]
}

@test "examples/logic.rw" {
   run $BIN examples/logic.rw
   [ "$status" -eq 0 ]
}

@test "examples/functions.rw" {
   run $BIN examples/functions.rw
   [ "$status" -eq 0 ]
}

@test "examples/imports/main.rw" {
   run $BIN examples/imports/main.rw
   [ "$status" -eq 0 ]
}

@test "Failed assertion" {
   run $BIN <(echo "assert false")
   [ "$status" -eq 1 ]
}

@test "wildcard" {
    run $BIN <<EOF
let _ => z
assert (a b) = z
EOF
    [ "$status" -eq 0 ]
}
