# `ortho_vec_derive`

A derive macro to create orthogonal vectors based on your structs, to allow for better CPU cache usage.

[![Crates.io](https://img.shields.io/crates/v/ortho_vec_derive)](https://crates.io/crates/ortho_vec_derive)
[![Last commit](https://img.shields.io/github/last-commit/yonatan-linik/ortho-vec-derive)](https://github.com/yonatan-linik/ortho-vec-derive/commits/main)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/yonatan-linik/ortho-vec-derive/rust.yml?branch=main)](https://github.com/yonatan-linik/ortho-vec-derive/actions)
![License: MIT](https://img.shields.io/crates/l/ortho_vec_derive)

## Motivation

Using a `Vec` of a large, complex `struct`, can slow down your run-time.&nbsp;
This can happen, when you don't use all of the fields of the struct when iterating over the vector.

For example let's imagine we have the following `struct`:

```rust
struct LargeStruct {
    speed: f32,
    location: f32,
    more_data: [u64; 10],
}
```

A `Vec<LargeStruct>` will look like this in memory:

```txt
   4B        4B         80B       4B        4B         80B 
[[speed] [location] [more_data] [speed] [location] [more_data]]
 ^                              ^
 |                              |
 first object starts here       second object starts here
```

Now we want to loop over the object in this `Vec` and update the location based on the speed:

```rust
struct LargeStruct {
    speed: f32,
    location: f32,
    more_data: [u64; 10],
}

let mut large_structs = vec![LargeStruct {speed: -1.2, location: 7.3, more_data: [0; 10]}];

for large_struct in large_structs.iter_mut() {
    large_struct.location += large_struct.speed;
}
```

The CPU will read the location in memory for `.speed` and `.location` but it will also cache the next bytes in memory.&nbsp;
How many bytes are cached can change based on your specific CPU.&nbsp;
What this means is, that we will read a lot of non required memory (`.more_data`) into cache.&nbsp;
In turn, the read of `.speed` for the next object we will have to go directly into RAM again, which is 10-100 times slower than cache.

To prevent these cache misses from happening and slowing down the program it is common to use a `Vec` per field in the struct, this happens such that the same index in all `Vec`-s represents the same object.

Writing the code for and managing a few `Vec`-s this way can be a bit complicated and require a lot of boilerplate code.

The `OrthoVec` derive macro tries to solve these issues.

## Supported API

### `Vec`

+ `into_ortho()` - Convert the `Vec` to its ortho version.

### ortho-`Vec`

All of these are the same as the `Vec` methods
+ `iter()`
+ `iter_mut()`
+ `into_iter()`
+ `len()`
+ `push()`
+ `pop()`
+ `clear()`
+ `reverse()`
+ `shrink_to_fit()`

The only caveat is that, when iterating with `iter_mut()`, you get a `struct` that contains a `&mut` to each inner `Vec`.&nbsp;
To use it you have to dereference it by adding a `*` prefix.

## Examples

Any named struct (for now - should add support for tuple-like in the future):

### `#[derive(OrthoVec)]`

```rust
use ortho_vec_derive::prelude::*;

#[derive(OrthoVec)]
struct WeirdStruct<'a, T: Send>
where
    T: std::fmt::Debug,
{
    a: i32,
    b: &'a f32,
    c: T,
}

fn main () {
    let mut ortho_vec = vec![
        WeirdStruct {
            a: 3,
            b: &4.2,
            c: "nice",
        },
        WeirdStruct {
            a: 6,
            b: &24.2,
            c: "hello",
        },
    ]
    .into_ortho();

    ortho_vec.push(WeirdStruct {
        a: 7,
        b: &8.123,
        c: "push",
    });

    for ws in ortho_vec.iter_mut() {
        *ws.a += 3;
    }

    for ws in ortho_vec.iter() {
        println!("b = {:?}", ws.b);
    }
}
```

## Results

Results may vary between use-cases and platforms.&nbsp;
Running a small benchmark (which is in the crate's repo) we can see a maximum of around 6X speedup, this is at 1M/10M but at sizes like 10K/100K we can also see speedup of around 2-3X.&nbsp;
It is recommended that you just try to bench the 2 versions of the code against each other, this way you can be sure this is working in your case.

## Notes

The macro can't be used on a struct defined inside a function for now.
