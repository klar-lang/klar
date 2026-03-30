---
title: "Klar Language Specification"
version: "0.1.0"
category: "reference"
keywords: ["syntax", "types", "functions", "modules", "concurrency", "schema"]
ai_context: true
last_updated: "2026-03-30"
---

# Klar Language Specification

> This file is designed for AI consumption. Paste into a system prompt, .cursorrules, or Claude Project knowledge to enable AI code generation for Klar.

## Types

### Primitives
- `Int` — 64-bit signed integer. Default: `0`
- `Float` — 64-bit IEEE 754. Default: `0.0`
- `Bool` — `true` or `false`. Default: `false`
- `String` — UTF-8, immutable. Default: `""`
- `Byte` — 0–255. Default: `0`
- `Unit` — Zero-sized, like void. Written `()`

### Composite
- `struct Name { field: Type, field: Type = default }`
- `enum Name { Variant, Variant(field: Type) }`
- `List[T]` — `[1, 2, 3]`
- `Map[K, V]` — `{"a": 1}`
- `Set[T]` — `Set.from([1, 2])`

### Option (no null)
- `T?` is sugar for `Option[T]`
- `fn find(id: Id) -> User?` — returns `some(user)` or `none`
- `let x = expr else { fallback }` — unwrap or execute fallback

### Result (no exceptions)
- `T ! E` is sugar for `Result[T, E]`
- `fn read(p: String) -> String ! IoError`
- `expr?` — propagate error to caller
- `expr catch err { handle }` — handle error inline

## Syntax

### Bindings
```
let x = value          // immutable
let mut x = value      // mutable
```

### Functions
```
fn name(a: Type, b: Type) -> ReturnType { body }
|args| expression      // anonymous function / closure
```

### Control Flow
```
if cond { } else { }                    // expression (returns value)
match value { Pattern => expr }         // exhaustive, compiler-enforced
for item in collection { }             // iteration (the only way)
for i, item in list { }               // with index
for key, val in map { }               // map iteration
loop { }                               // infinite loop (break to exit)
```

### Error Handling
```
let val = fallible_fn()?               // propagate error
let val = fallible_fn() catch err { }  // handle error
let val = optional else { fallback }   // unwrap option
```

### Modules
```
use std.json
use std.http.{Request, Response}
use app.module.{Item}
```

### String Interpolation
```
"Hello {name}, you are {age} years old"
```

### Pipe Operator
```
value |> transform |> format
```

## Concurrency

```
// Sequential (await is implicit)
let user = http.get("/users/{id}")?

// Parallel
let (a, b, c) = parallel {
    fetch_a()?
    fetch_b()?
    fetch_c()?
}

// Channels
let (send, recv) = channel[T](buffer: 10)
spawn { send.emit(value) }
let msg = recv.next()?
```

## Schema System

```
@schema
struct User {
    name: String       @min_len(1) @max_len(100)
    email: String      @format(email)
    age: Int           @range(0, 150)
    role: Role = Role.Member
}

// Auto-generated:
// json.encode(user) -> String
// json.decode[User](str) -> User ! DecodeError
// User.validate(data) -> User ! ValidationError
// User.schema() -> JSON Schema / OpenAPI
// User.ts_type() -> TypeScript type
```

## Traits

```
trait Printable {
    fn to_string(self) -> String
}

impl Printable for User {
    fn to_string(self) -> String { "{self.name}" }
}

fn print_all[T: Printable](items: List[T]) { }
```

## Testing

```
test name {
    assert expr == expected
    assert_eq(a, b)
    assert_err(fallible_fn())
}

forall x: Int, y: Int {
    assert add(x, y) == add(y, x)
}
```

## Standard Library

| Module | Key functions |
|--------|--------------|
| std.string | split, join, trim, contains, replace, starts_with, ends_with |
| std.list | map, filter, reduce, find, sort, reverse, take, drop, zip, flatten |
| std.map | get, set, remove, keys, values, merge, filter |
| std.set | add, remove, contains, union, intersection, difference |
| std.json | encode, decode, pretty_print |
| std.http | get, post, put, delete, serve, Router, Request, Response |
| std.io | read_file, write_file, stdin, stdout, File, Path |
| std.time | now, parse, format, Duration, DateTime |
| std.crypto | hash_sha256, hash_bcrypt, random_bytes, uuid |
| std.sql | connect, query, execute, Transaction, Pool |
| std.test | assert, assert_eq, assert_err, describe, it, mock |
| std.log | debug, info, warn, error, with_context |
| std.env | get, require, load_dotenv |

## Common Patterns

### HTTP Handler
```
use std.http.{Router, Request, Response, serve}

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

### CLI Tool
```
use std.env
use std.io

fn main() ! AppError {
    let args = env.args()
    match args.get(1) {
        some("help") => println("Usage: mytool <command>")
        some(cmd) => run_command(cmd)?
        none => println("No command provided")
    }
}
```

## Anti-Patterns (DO NOT generate)

- No `null`, `nil`, `undefined`, `None` — use `Option[T]`
- No `try/catch/throw` — use `Result[T, E]` with `?` and `catch`
- No `var`, `const`, `:=` — use `let` and `let mut`
- No `while` loops — use `for item in collection` or `loop`
- No `class` or inheritance — use `struct` + `trait` + `impl`
- No ternary `?:` — use `if/else` expression
- No `.forEach()` or `.map()` method chains — use `for` or `|>` pipe

## Keywords (29 total)

```
let mut fn struct enum trait impl use
if else match for in loop break return
true false and or not pub priv test
spawn parallel catch async unsafe
```
