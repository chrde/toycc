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

    RUST_BACKTRACE=1 ./target/debug/toycc "$input" 2> tmp.error || exit

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

# assert 0 '{ return 0; }'
# assert 42 '{ return 42; }'

# assert 21 '{ return 5+20-4; }'
# assert 41 '{ return  12 + 34 - 5; }'

# assert 47 '{ return 5+6*7; }'
# assert 15 '{ return 5*(9-6); }'
# assert 4 '{ return (3+5)/2; }'

# assert 10 '{ return -10+20; }'
# assert 10 '{ return - -10; }'
# assert 10 '{ return - - +10; }'

# assert 0 '{ return 0==1; }'
# assert 1 '{ return 42==42; }'
# assert 1 '{ return 0!=1; }'
# assert 0 '{ return 42!=42; }'

# assert 1 '{ return 0<1; }'
# assert 0 '{ return 1<1; }'
# assert 0 '{ return 2<1; }'
# assert 1 '{ return 0<=1; }'
# assert 1 '{ return 1<=1; }'
# assert 0 '{ return 2<=1; }'

# assert 1 '{ return 1>0; }'
# assert 0 '{ return 1>1; }'
# assert 0 '{ return 1>2; }'
# assert 1 '{ return 1>=0; }'
# assert 1 '{ return 1>=1; }'
# assert 0 '{ return 1>=2; }'

# assert 3 '{ 1; 2; return 3; }'

# assert 3 '{ a = 3; return a; }'
# assert 8 '{ a=3; z=5; return a+z; }'
# assert 6 '{ a=b=3; return a+b; }'
# assert 3 '{ foo=3; return foo; }'
# assert 8 '{ foo123=3; bar=5; return foo123+bar; }'

# assert 1 '{ return 1; 2; 3; }'
# assert 2 '{ 1; return 2; 3; }'
# assert 3 '{ 1; 2; return 3; }'

# assert 3 '{ {1; {2;} return 3;} }'
# assert 5 '{ ;;; return 5; }'

# assert 3 '{ if (0) return 2; return 3; }'
# assert 3 '{ if (1-1) return 2; return 3; }'
# assert 2 '{ if (1) return 2; return 3; }'
# assert 2 '{ if (2-1) return 2; return 3; }'
# assert 4 '{ if (0) { 1; 2; return 3; } else { return 4; } }'
# assert 3 '{ if (1) { 1; 2; return 3; } else { 1; return 4; 3; } }'

# assert 0 '{ for (i=0; i<=10; i=i+1) {} }'
# assert 55 '{ i=0; j=0; for (i=0; i<=10; i=i+1) j=i+j; return j; }'
# assert 3 '{ for (;;) {return 3;} return 5; }'

# assert 10 '{ i=0; while(i<10) { i=i+1; } return i; }'

# assert 3 '{ x=3; return *&x; }'
# assert 3 '{ x=3; y=&x; z=&y; return **z; }'
# assert 5 '{ x=3; y=5; return *(&x+8); }'
assert 3 '{ x=3; y=5; return *(&y-8); }'
# assert_err $'error: `expected number`\n12 + 34 - 5 - -\n               ^' '12 + 34 - 5 - -'

echo OK
