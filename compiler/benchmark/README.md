# Klar AI Correctness Benchmark

500 standardized programming tasks to measure AI code generation correctness across languages.

## Structure

```
benchmark/
├── tasks/           # Task definitions (YAML)
│   ├── 001_hello_world.yaml
│   ├── 002_fizzbuzz.yaml
│   └── ...
├── runner/          # Benchmark automation
│   └── run.py       # Feed tasks to AI, compile, test
└── results/         # Benchmark results (generated)
```

## Task Format

Each task is a YAML file:

```yaml
id: "001"
title: "Hello World"
category: "basics"
difficulty: "easy"
prompt: |
  Write a Klar function called `hello` that takes a name (String)
  and returns "Hello, {name}!" as a String.
test_cases:
  - input: '"World"'
    expected: '"Hello, World!"'
  - input: '"Klar"'
    expected: '"Hello, Klar!"'
```

## Running

```bash
python benchmark/runner/run.py --language klar --model claude-sonnet
python benchmark/runner/run.py --language typescript --model claude-sonnet
```

## Categories

| Category | Count | Examples |
|----------|-------|---------|
| Basics | 30 | Hello world, arithmetic, string ops |
| String manipulation | 50 | Reverse, validate email, CSV parse |
| Data structures | 50 | Sort, merge, stack, queue |
| HTTP/API | 75 | REST endpoint, middleware, auth |
| Database | 50 | CRUD, migrations, transactions |
| File I/O | 40 | Read/write, config parse, streaming |
| JSON | 50 | Parse, transform, validate |
| Error handling | 50 | Recovery, retry, fallback |
| Concurrency | 35 | Parallel fetch, producer-consumer |
| CLI tools | 30 | Arg parse, prompts, progress |
| Full apps | 40 | Todo API, URL shortener, chat |
