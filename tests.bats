#!/usr/bin/env bats

BIN='./rw'

@test "examples/cycle.rw" {
   run $BIN examples/cycle.rw
   [ "$status" -eq 0 ]
}

@test "examples/algebra.rw" {
   run $BIN examples/algebra.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/peano.rw" {
   run $BIN examples/peano.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/lists.rw" {
   run $BIN examples/lists.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/rules.rw" {
   run $BIN examples/rules.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/mappings.rw" {
   run $BIN examples/mappings.rw
   [ "$status" -eq 0 ]
}

@test "examples/operators.rw" {
   run $BIN examples/operators.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/patterns.rw" {
   run $BIN examples/patterns.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/logic.rw" {
   run $BIN examples/logic.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/functions.rw" {
   run $BIN examples/functions.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/magma.rw" {
   run $BIN examples/magma.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}


@test "examples/or.rw" {
   run $BIN examples/or.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "examples/imports/main.rw" {
   run $BIN examples/imports/main.rw
   for line in "${lines[@]}"; do
      [ "$line" = "ok" ]
   done
   [ "$status" -eq 0 ]
}

@test "merge rules" {
   run $BIN <<EOF
let a => x
let b|c => x
rules

let _ => x
rules
EOF
   [ "${lines[0]}" = "a|b|c => x" ]
   [ "${lines[1]}" = "_ => x" ]
   [ "$status" -eq 0 ]
}
