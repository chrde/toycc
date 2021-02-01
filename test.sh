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
if [[ "$?" != 0 ]]; then exit 1;fi

assert 0 '0;'
assert 42 '42;'

assert 21 '5+20-4;'
assert 41 ' 12 + 34 - 5;'

assert 47 '5+6*7;'
assert 15 '5*(9-6);'
assert 4 '(3+5)/2;'

assert 10 '-10+20;'
assert 10 '- -10;'
assert 10 '- - +10;'

assert 0 '0==1;'
assert 1 '42==42;'
assert 1 '0!=1;'
assert 0 '42!=42;'

assert 1 '0<1;'
assert 0 '1<1;'
assert 0 '2<1;'
assert 1 '0<=1;'
assert 1 '1<=1;'
assert 0 '2<=1;'

assert 1 '1>0;'
assert 0 '1>1;'
assert 0 '1>2;'
assert 1 '1>=0;'
assert 1 '1>=1;'
assert 0 '1>=2;'

assert 3 '1; 2; 3;'

assert 3 'a = 3; a;'
assert 8 'a=3; z=5; a+z;'
assert 6 'a=b=3; a+b;'
assert 3 'foo=3; foo;'
assert 8 'foo123=3; bar=5; foo123+bar;'
# assert_err $'error: `expected number`\n12 + 34 - 5 - -\n               ^' '12 + 34 - 5 - -'

echo OK
