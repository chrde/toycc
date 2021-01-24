#!/usr/bin/env bash
assert() {
    rm -f tmp.s
    expected="$1"
    input="$2"

    ./target/debug/toycc "$input" > tmp.s || exit
    gcc -static -o tmp tmp.s
    ./tmp
    actual="$?"

    if [ "$actual" = "$expected" ]; then
        echo "$input => $actual"
    else
        echo "$input => $expected expected, but got $actual"
        exit 1
    fi
}

assert_err() {
    rm -f tmp.error
    expected="$1"
    input="$2"

    ./target/debug/toycc "$input" 2> tmp.error || exit

    if [ "$?" != 0 ]; then
        echo "$input => did not fail"
        exit 1
    fi

    actual=$(cat tmp.error)

    if [ "$actual" = "$expected" ]; then
        echo "[OK]$input"
    else
        printf "Expected\t%s\nActual\t%s\n" "$expected" "$actual"
        exit 1
    fi
}

cargo build
assert 0 0
assert 42 42
assert 21 '5+20-4'
assert 41 ' 12 + 34 - 5'
assert_err $'error: `expected number`\n12 + 34 - 5 - -\n               ^' '12 + 34 - 5 - -'

echo OK
