#!/usr/bin/env bats

BIN='./rw'

@test "examples/cycle.rw" {
   run $BIN examples/cycle.rw
   [ "$output" = "true" ]
	[ "$status" -eq 0 ]
}

@test "examples/algebra.rw" {
   run $BIN examples/algebra.rw
   [ "$output" = "true" ]
	[ "$status" -eq 0 ]
}

@test "examples/peano.rw" {
   run $BIN examples/peano.rw
   for line in "${lines[@]}"; do
      [ "$line" = "true" ]
   done
   [ ${#lines[@]} -eq 2 ]
	[ "$status" -eq 0 ]
}

@test "examples/lists.rw" {
   run $BIN examples/lists.rw
   for line in "${lines[@]}"; do
      [ "$line" = "true" ]
   done
   [ ${#lines[@]} -eq 2 ]
	[ "$status" -eq 0 ]
}

@test "examples/rules.rw" {
   run $BIN examples/rules.rw
   [ "$output" = "true" ]
	[ "$status" -eq 0 ]
}

@test "examples/variables.rw" {
   run $BIN examples/variables.rw
   for line in "${lines[@]}"; do
      [ "$line" = "true" ]
   done
   [ ${#lines[@]} -eq 3 ]
	[ "$status" -eq 0 ]
}

@test "examples/operators.rw" {
   run $BIN examples/operators.rw
   [ "$output" = "true" ]
	[ "$status" -eq 0 ]
}

@test "examples/logic.rw" {
   run $BIN examples/logic.rw
   [ "$output" = "true" ]
   [ "$status" -eq 0 ]
}

@test "examples/functions.rw" {
   run $BIN examples/functions.rw
   [ "${lines[0]}" = "false" ]
   [ "${lines[1]}" = "true" ]
   [ "$status" -eq 0 ]
}

@test "Failed assertion" {
   run $BIN <(echo "assert false")
   [ "$status" -eq 1 ]
}

@test "wildcard" {
    run $BIN <<EOF
let _ => z
eval (a b) = z
EOF
    [ "$output" = "true" ]
    [ "$status" -eq 0 ]
}
