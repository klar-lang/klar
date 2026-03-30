# Klar Language Specification v0.1.0

> Paste this file into your AI system prompt, Claude Project, or .cursorrules to generate correct Klar code.

## Overview

Klar is a statically typed language compiled to JavaScript (native and WASM targets coming). It has NO null, NO exceptions, and exactly ONE way to do each operation.

## Types

**Primitives:** `Int` (64-bit), `Float` (64-bit), `Bool`, `String` (UTF-8, immutable), `Byte`, `Unit`

**Collections:** `List[T]` = `[1, 2, 3]`, `Map[K, V]` = `{"a": 1}`, `Set[T]` = `Set.from([1, 2])`

**Option (no null):** `T?` — values that may be absent. Must unwrap with `else`.
```
fn find(id: Id) -> User? { }
let user = find(id) else { return err("not found") }
```

**Result (no exceptions):** `T ! E` — functions that can fail. Must handle with `?` or `catch`.
```
fn read(path: String) -> String ! IoError { }
let text = read("file.txt")?              // propagate error
let text = read("file.txt") catch err { "default" }  // handle inline
```

## Syntax

```
let x = value              // immutable binding
let mut x = value          // mutable binding
fn name(a: Type) -> Type { body }  // function (last expr is return value)
|args| expression          // closure
if cond { } else { }      // conditional (expression)
for item in collection { } // iteration (the ONLY loop for collections)
for i, item in list { }   // with index
loop { break }             // infinite loop
match val { Pat => expr }  // pattern match (must be exhaustive)
expr?                       // error propagation
expr catch err { }         // error handling
value |> fn1 |> fn2        // pipe
"Hello {name}"             // string interpolation
use std.module.{Item}      // imports
@annotation                // annotations (@schema, @min_len, etc.)
```

## Declarations

```
struct Name { field: Type }
struct Name { field: Type @annotation(arg) }

enum Name { Variant, Variant(field: Type) }

trait Name { fn method(self) -> Type }
impl Trait for Type { fn method(self) -> Type { body } }

test name { assert expr == expected }
```

## Standard Library

| Module | Key functions |
|--------|--------------|
| `std.string` | split, join, trim, contains, replace, starts_with, ends_with, to_upper, to_lower, length, chars, repeat, slice |
| `std.list` | map, filter, reduce, find, any, all, sort, reverse, take, drop, chunk, zip, flatten, unique, length, push, concat, includes |
| `std.map` | get, set, remove, keys, values, entries, merge, filter, map_values, size, contains |
| `std.set` | from, add, remove, contains, union, intersection, difference, is_subset, size |
| `std.json` | encode, decode, pretty_print |
| `std.math` | abs, min, max, floor, ceil, round, pow, sqrt, pi, e, clamp, random |
| `std.io` | read_file, write_file, file_exists, read_lines |
| `std.env` | get, require, args |
| `std.time` | now, timestamp, format, parse |
| `std.crypto` | uuid, random_bytes, hash_sha256 |
| `std.log` | debug, info, warn, error |

**Usage:** `use std.json` then `json.encode(data)`. Or `use std.json.{encode, decode}` then `encode(data)`.

## Common Patterns

### HTTP Handler
```
use std.http.{Router, Request, Response, serve}

@schema
struct User {
    name: String @min_len(1)
    email: String @format(email)
}

fn main() ! ServerError {
    let router = Router.new()
        |> Router.get("/users/{id}", get_user)
        |> Router.post("/users", create_user)
    serve(router, port: 3000)?
}

fn get_user(req: Request) -> Response ! AppError {
    let id = req.param("id")
    let user = db.find[User](id)?
    Response.json(user)
}
```

### Data Processing
```
use std.list
use std.json

fn process(items: List[Int]) -> List[Int] {
    items
        |> list.filter(|x| x > 0)
        |> list.map(|x| x * 2)
        |> list.sort(|a, b| a - b)
}
```

### CLI Tool
```
use std.env
use std.io

fn main() {
    let args = env.args()
    match list.get(args, 0) {
        "help" => println("Usage: mytool <cmd>")
        "run" => run_command()
        _ => println("Unknown command")
    }
}
```

### Error Handling
```
fn safe_divide(a: Float, b: Float) -> Float ! MathError {
    if b == 0.0 {
        return err(MathError.DivisionByZero)
    }
    a / b
}

fn main() {
    let result = safe_divide(10.0, 3.0) catch err {
        println("Error: {err}")
        0.0
    }
}
```

### Struct with Schema
```
@schema
struct Config {
    host: String
    port: Int @range(1, 65535)
    debug: Bool
}

// Auto-generated: json.encode, json.decode[Config], Config.validate
```

## Anti-Patterns (DO NOT generate)

| Wrong | Correct | Why |
|-------|---------|-----|
| `null`, `nil`, `undefined`, `None` | `Option[T]` with `?` suffix | No null in Klar |
| `try { } catch { }`, `throw` | `Result[T, E]` with `?` and `catch` | No exceptions |
| `var x`, `const x`, `x := ` | `let x` or `let mut x` | One binding syntax |
| `while (cond)`, `do { } while` | `for item in collection` or `loop { }` | One iteration syntax |
| `class`, `extends`, `super` | `struct` + `trait` + `impl` | No inheritance |
| `x ? a : b` (ternary) | `if x { a } else { b }` | No ternary |
| `.forEach()`, `.map()` chains | `for` loop or `\|>` pipe with `list.map` | One way to iterate |
| `import`, `require`, `from` | `use module.{Item}` | One import syntax |
| `f"string"`, `` `template` `` | `"Hello {expr}"` | One interpolation syntax |

## Keywords (29)

```
let mut fn struct enum trait impl use
if else match for in loop break return
true false and or not pub priv test
spawn parallel catch async unsafe
```

## Toolchain

```
klar run <file>     # compile + execute
klar build <file>   # compile to .js
klar test <file>    # run test blocks
klar check <file>   # type-check only (--format json for AI)
klar fmt <file>     # format (--check for CI)
klar lex <file>     # show token stream
klar parse <file>   # show AST summary
```
