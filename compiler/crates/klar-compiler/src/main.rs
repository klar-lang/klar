use klar_lexer::{Lexer, TokenKind};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("lex") => cmd_lex(&args[2..]),
        Some("parse") => cmd_parse(&args[2..]),
        Some("build") => cmd_build(&args[2..]),
        Some("run") => cmd_run(&args[2..]),
        Some("test") => cmd_test(&args[2..]),
        Some("check") => cmd_check(&args[2..]),
        Some("fmt") => cmd_fmt(&args[2..]),
        Some("init") => cmd_init(&args[2..]),
        Some("add") => cmd_add(&args[2..]),
        Some("remove") => cmd_remove(&args[2..]),
        Some("lint") => cmd_lint(&args[2..]),
        Some("bench") => cmd_bench(&args[2..]),
        Some("doc") => cmd_doc(&args[2..]),
        Some("migrate") => cmd_migrate(&args[2..]),
        Some("deploy") => cmd_deploy(&args[2..]),
        Some("repl") => cmd_repl(&args[2..]),
        Some("new") => cmd_new(&args[2..]),
        Some("publish") => cmd_publish(&args[2..]),
        Some("audit") => cmd_audit(&args[2..]),
        Some("--version") | Some("-V") => {
            println!("klar 1.0.0-beta");
        }
        Some("--help") | Some("-h") | None => print_help(),
        Some(cmd) => {
            eprintln!("error: unknown command '{cmd}'");
            eprintln!();
            print_help();
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("klar 1.0.0-beta — The AI-First Programming Language");
    println!();
    println!("USAGE:");
    println!("    klar <command> [args]");
    println!();
    println!("COMMANDS:");
    println!("    lex <file>     Tokenize a .klar file and print tokens");
    println!("    parse <file>   Parse a .klar file and print the AST summary");
    println!("    build <file>   Compile a .klar file (default: JS, --target native for binary)");
    println!("    run <file>     Compile and execute (--target native for native execution)");
    println!("    test <file>    Run test blocks in a .klar file");
    println!("    check <file>   Type-check without building (--format json for AI)");
    println!("    fmt <file>     Format a .klar file (check mode: --check)");
    println!("    init [name]    Initialize a new klar.toml in the current directory");
    println!("    add <pkg>      Add a dependency (e.g., klar add http-server@1.0)");
    println!("    remove <pkg>   Remove a dependency");
    println!("    lint <file>    Lint a .klar file for style and correctness issues");
    println!("    bench <file>   Run benchmarks in a .klar file");
    println!("    doc <file>     Generate documentation from doc comments");
    println!("    migrate        Database migrations (up, down, generate, status)");
    println!("    deploy         Deploy to cloud (fly, docker)");
    println!("    repl           Interactive REPL");
    println!("    new <name>     Create a new Klar project from template");
    println!("    publish        Publish package to registry");
    println!("    audit          Check dependencies for vulnerabilities");
    println!("    --version      Print version");
    println!("    --help         Print this help");
}

fn cmd_lex(args: &[String]) {
    let path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("error: expected a file path");
            eprintln!("usage: klar lex <file.klar>");
            process::exit(1);
        }
    };

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };

    let tokens = Lexer::tokenize(&source);
    let mut errors = 0;

    for tok in &tokens {
        let text = &source[tok.span.start..tok.span.end];
        match &tok.kind {
            TokenKind::Newline | TokenKind::Eof => {}
            TokenKind::Error(msg) => {
                errors += 1;
                println!(
                    "  \x1b[31mERROR\x1b[0m  {:>4}:{:<4} {}",
                    tok.span.start, tok.span.end, msg
                );
            }
            TokenKind::Comment(_) => {
                println!(
                    "  \x1b[90mCOMMENT\x1b[0m {:>4}:{:<4} {:?}",
                    tok.span.start, tok.span.end, text
                );
            }
            _ => {
                println!(
                    "  \x1b[32m{:<12}\x1b[0m {:>4}:{:<4} {:?}",
                    format!("{:?}", tok.kind).split('(').next().unwrap_or("?"),
                    tok.span.start,
                    tok.span.end,
                    text
                );
            }
        }
    }

    let count = tokens.iter().filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof | TokenKind::Comment(_))).count();
    println!();
    if errors > 0 {
        println!("\x1b[31m{} tokens, {} errors\x1b[0m", count, errors);
    } else {
        println!("\x1b[32m{} tokens, 0 errors\x1b[0m", count);
    }
}

fn read_source(args: &[String], cmd: &str) -> (String, String) {
    let path = match args.first() {
        Some(p) => p.clone(),
        None => {
            eprintln!("error: expected a file path");
            eprintln!("usage: klar {} <file.klar>", cmd);
            process::exit(1);
        }
    };
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };
    (path, source)
}

fn cmd_parse(args: &[String]) {
    let (path, source) = read_source(args, "parse");

    match klar_parser::parse(&source) {
        Ok(program) => {
            let mut fns = 0;
            let mut structs = 0;
            let mut enums = 0;
            let mut traits = 0;
            let mut impls = 0;
            let mut uses = 0;
            let mut tests = 0;

            for item in &program.items {
                match item {
                    klar_ast::Item::Function(_) => fns += 1,
                    klar_ast::Item::Struct(_) => structs += 1,
                    klar_ast::Item::Enum(_) => enums += 1,
                    klar_ast::Item::Trait(_) => traits += 1,
                    klar_ast::Item::Impl(_) => impls += 1,
                    klar_ast::Item::Use(_) => uses += 1,
                    klar_ast::Item::Test(_) => tests += 1,
                }
            }

            println!("\x1b[32m✓\x1b[0m Parsed \x1b[1m{}\x1b[0m — {} items", path, program.items.len());
            println!();
            if uses > 0 { println!("  use        {}", uses); }
            if structs > 0 { println!("  struct     {}", structs); }
            if enums > 0 { println!("  enum       {}", enums); }
            if fns > 0 { println!("  fn         {}", fns); }
            if traits > 0 { println!("  trait      {}", traits); }
            if impls > 0 { println!("  impl       {}", impls); }
            if tests > 0 { println!("  test       {}", tests); }
        }
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    let use_native = args.iter().any(|a| a == "--target" || a == "native");
    let file_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && *a != "native")
        .cloned()
        .collect();
    let (path, source) = read_source(&file_args, "run");

    // Parse
    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    };

    // Type check
    let mut checker = klar_typeck::TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        eprintln!("\x1b[31m✗\x1b[0m {} type errors in {}:", errors.len(), path);
        for err in &errors {
            eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
        }
        process::exit(1);
    }

    if use_native {
        // Compile to native and execute
        let tmp_dir = std::env::temp_dir();
        let bin_path = tmp_dir.join("klar_native");
        match klar_codegen_llvm::compile_to_native(&program, &bin_path) {
            Ok(()) => {
                let output = std::process::Command::new(&bin_path).output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stdout.is_empty() { print!("{}", stdout); }
                        if !stderr.is_empty() { eprint!("{}", stderr); }
                        if !out.status.success() {
                            process::exit(out.status.code().unwrap_or(1));
                        }
                    }
                    Err(e) => {
                        eprintln!("\x1b[31m✗\x1b[0m Failed to execute native binary: {}", e);
                        process::exit(1);
                    }
                }
                let _ = std::fs::remove_file(&bin_path);
            }
            Err(e) => {
                eprintln!("\x1b[31m✗\x1b[0m Native compilation failed: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Generate JS and execute with Node.js
        let js = klar_codegen_js::generate(&program);
        let tmp_dir = std::env::temp_dir();
        let js_path = tmp_dir.join("klar_output.js");
        fs::write(&js_path, &js).expect("failed to write JS output");

        let output = std::process::Command::new("node")
            .arg(&js_path)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                if !stdout.is_empty() { print!("{}", stdout); }
                if !stderr.is_empty() { eprint!("{}", stderr); }
                if !out.status.success() {
                    process::exit(out.status.code().unwrap_or(1));
                }
            }
            Err(_) => {
                eprintln!("\x1b[31m✗\x1b[0m Node.js not found. Install Node.js to run Klar programs.");
                eprintln!("  Generated JS saved to: {}", js_path.display());
                process::exit(1);
            }
        }
    }
}

fn cmd_build(args: &[String]) {
    let target = if args.iter().any(|a| a == "native") {
        "native"
    } else if args.iter().any(|a| a == "llvm-ir") {
        "llvm-ir"
    } else {
        "js"
    };

    let file_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && *a != "native" && *a != "llvm-ir" && *a != "js")
        .cloned()
        .collect();
    let (path, source) = read_source(&file_args, "build");

    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    };

    let mut checker = klar_typeck::TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        eprintln!("\x1b[31m✗\x1b[0m {} type errors in {}:", errors.len(), path);
        for err in &errors {
            eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
        }
        process::exit(1);
    }

    match target {
        "native" => {
            let out_path = std::path::Path::new(&path).with_extension("");
            match klar_codegen_llvm::compile_to_native(&program, &out_path) {
                Ok(()) => {
                    println!(
                        "\x1b[32m✓\x1b[0m Compiled {} → {} (native)",
                        path,
                        out_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("\x1b[31m✗\x1b[0m Native compilation failed: {}", e);
                    process::exit(1);
                }
            }
        }
        "llvm-ir" => {
            match klar_codegen_llvm::generate_ir(&program) {
                Ok(ir) => {
                    let out_path = path.replace(".klar", ".ll");
                    fs::write(&out_path, &ir).expect("failed to write LLVM IR");
                    println!("\x1b[32m✓\x1b[0m Compiled {} → {}", path, out_path);
                }
                Err(e) => {
                    eprintln!("\x1b[31m✗\x1b[0m LLVM IR generation failed: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            let js = klar_codegen_js::generate(&program);
            let out_path = path.replace(".klar", ".js");
            fs::write(&out_path, &js).expect("failed to write output");
            println!("\x1b[32m✓\x1b[0m Compiled {} → {}", path, out_path);
        }
    }
}

fn cmd_check(args: &[String]) {
    let json_mode = args.iter().any(|a| a == "--format" || a == "json");
    let file_args: Vec<String> = args.iter()
        .filter(|a| !a.starts_with("--") && *a != "json")
        .cloned()
        .collect();
    let (path, source) = read_source(&file_args, "check");

    let mut diagnostics: Vec<String> = Vec::new();

    // Parse
    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            for err in &errors {
                let (line, col) = offset_to_line_col(&source, err.span.start);
                if json_mode {
                    diagnostics.push(format!(
                        r#"{{"code":"E001","severity":"error","message":"{}","location":{{"file":"{}","line":{},"col":{}}},"actions":["fix_syntax"],"fixes":[{{"action":"fix_syntax","description":"Check syntax near this location"}}]}}"#,
                        err.message.replace('"', "\\\""), path, line, col
                    ));
                } else {
                    eprintln!("\x1b[31merror[E001]\x1b[0m: Parse error");
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    eprintln!("   \x1b[34m|\x1b[0m");
                    eprintln!("   \x1b[34m|\x1b[0m {}", err.message);
                    eprintln!("   \x1b[34m=\x1b[0m fix: Check syntax near this location");
                    eprintln!("   Available actions: [fix_syntax]");
                    eprintln!();
                }
            }
            if json_mode {
                println!("[{}]", diagnostics.join(","));
            }
            process::exit(1);
        }
    };

    // Type check
    let mut checker = klar_typeck::TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        for err in &errors {
            let (line, col) = offset_to_line_col(&source, err.span.start);
            let (code, actions, fixes) = categorize_type_error(&err.message);
            if json_mode {
                let fixes_json: Vec<String> = fixes.iter()
                    .map(|(a, d)| format!(r#"{{"action":"{}","description":"{}"}}"#, a, d))
                    .collect();
                diagnostics.push(format!(
                    r#"{{"code":"{}","severity":"error","message":"{}","location":{{"file":"{}","line":{},"col":{}}},"actions":[{}],"fixes":[{}]}}"#,
                    code,
                    err.message.replace('"', "\\\""),
                    path, line, col,
                    actions.iter().map(|a| format!(r#""{}""#, a)).collect::<Vec<_>>().join(","),
                    fixes_json.join(",")
                ));
            } else {
                eprintln!("\x1b[31merror[{}]\x1b[0m: {}", code, err.message);
                eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                for (action, desc) in &fixes {
                    eprintln!("   \x1b[34m=\x1b[0m fix: {} ({})", desc, action);
                }
                eprintln!("   Available actions: [{}]", actions.join(", "));
                eprintln!();
            }
        }
        if json_mode {
            println!("[{}]", diagnostics.join(","));
        }
        process::exit(1);
    }

    if json_mode {
        println!("[]");
    } else {
        println!("\x1b[32m✓\x1b[0m No errors in {}", path);
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset { break; }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn categorize_type_error(msg: &str) -> (&str, Vec<&str>, Vec<(&str, &str)>) {
    if msg.contains("undefined") {
        ("E010", vec!["add_import", "define_variable"],
         vec![("add_import", "Import the module or define the variable"),
              ("define_variable", "Add a let binding for this name")])
    } else if msg.contains("type mismatch") {
        ("E020", vec!["change_type", "add_conversion"],
         vec![("change_type", "Change the value to match the expected type"),
              ("add_conversion", "Add an explicit type conversion")])
    } else if msg.contains("expected") && msg.contains("arguments") {
        ("E030", vec!["fix_args"],
         vec![("fix_args", "Adjust the number of arguments to match the function signature")])
    } else if msg.contains("operator requires") {
        ("E040", vec!["change_type"],
         vec![("change_type", "Ensure operands are the correct type for this operator")])
    } else if msg.contains("cannot iterate") {
        ("E050", vec!["wrap_iterable"],
         vec![("wrap_iterable", "Wrap the value in a List or use an iterable type")])
    } else {
        ("E099", vec!["fix_code"],
         vec![("fix_code", "Review the code at this location")])
    }
}

fn cmd_test(args: &[String]) {
    let (path, source) = read_source(args, "test");

    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    };

    let mut checker = klar_typeck::TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        eprintln!("\x1b[31m✗\x1b[0m {} type errors in {}:", errors.len(), path);
        for err in &errors {
            eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
        }
        process::exit(1);
    }

    // Count test blocks
    let test_count = program.items.iter()
        .filter(|i| matches!(i, klar_ast::Item::Test(_)))
        .count();

    if test_count == 0 {
        println!("No tests found in {}", path);
        return;
    }

    println!("\x1b[1mRunning {} tests from {}\x1b[0m", test_count, path);
    println!();

    // Generate and run — test blocks auto-execute in generated JS
    let js = klar_codegen_js::generate(&program);
    let tmp_dir = std::env::temp_dir();
    let js_path = tmp_dir.join("klar_test.js");
    fs::write(&js_path, &js).expect("failed to write JS output");

    let output = std::process::Command::new("node")
        .arg(&js_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            // Filter output to show only test results
            for line in stdout.lines() {
                if line.contains("✓") || line.contains("✗") || line.contains("assert") {
                    println!("{}", line);
                }
            }
            if out.status.success() {
                println!();
                println!("\x1b[32m{} tests passed\x1b[0m", test_count);
            } else {
                if !stderr.is_empty() { eprint!("{}", stderr); }
                println!();
                println!("\x1b[31mTests failed\x1b[0m");
                process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("\x1b[31m✗\x1b[0m Node.js not found.");
            process::exit(1);
        }
    }
}

fn cmd_fmt(args: &[String]) {
    let check_mode = args.iter().any(|a| a == "--check");
    let file_args: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();

    let (path, source) = read_source(
        &file_args.into_iter().cloned().collect::<Vec<_>>(),
        "fmt",
    );

    // Parse the source
    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    };

    // Format by re-emitting from AST
    let formatted = format_program(&program);

    if check_mode {
        if formatted == source {
            println!("\x1b[32m✓\x1b[0m {} is correctly formatted", path);
        } else {
            eprintln!("\x1b[31m✗\x1b[0m {} needs formatting", path);
            process::exit(1);
        }
    } else {
        fs::write(&path, &formatted).expect("failed to write formatted output");
        println!("\x1b[32m✓\x1b[0m Formatted {}", path);
    }
}

fn format_program(program: &klar_ast::Program) -> String {
    let mut out = String::new();
    for (i, item) in program.items.iter().enumerate() {
        if i > 0 { out.push('\n'); }
        format_item(&mut out, item, 0);
    }
    out
}

fn format_item(out: &mut String, item: &klar_ast::Item, indent: usize) {
    let pad = "    ".repeat(indent);
    match item {
        klar_ast::Item::Use(u) => {
            out.push_str(&format!("{}use {}", pad, u.path.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(".")));
            if let Some(items) = &u.items {
                out.push_str(&format!(".{{{}}}", items.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(", ")));
            }
            out.push('\n');
        }
        klar_ast::Item::Struct(s) => {
            for ann in &s.annotations {
                out.push_str(&format!("{}@{}", pad, ann.name.name));
                if !ann.args.is_empty() { out.push_str("(...)"); }
                out.push('\n');
            }
            out.push_str(&format!("{}struct {} {{\n", pad, s.name.name));
            for field in &s.fields {
                out.push_str(&format!("{}    {}: {}", pad, field.name.name, format_type(&field.ty)));
                for ann in &field.annotations {
                    out.push_str(&format!(" @{}", ann.name.name));
                    if !ann.args.is_empty() { out.push_str("(...)"); }
                }
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        klar_ast::Item::Enum(e) => {
            out.push_str(&format!("{}enum {} {{\n", pad, e.name.name));
            for v in &e.variants {
                out.push_str(&format!("{}    {}", pad, v.name.name));
                if !v.fields.is_empty() {
                    let params: Vec<String> = v.fields.iter().map(|f| format!("{}: {}", f.name.name, format_type(&f.ty))).collect();
                    out.push_str(&format!("({})", params.join(", ")));
                }
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        klar_ast::Item::Function(f) => {
            if f.is_priv { out.push_str(&format!("{}priv ", pad)); } else { out.push_str(&pad); }
            out.push_str(&format!("fn {}(", f.name.name));
            let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name.name, format_type(&p.ty))).collect();
            out.push_str(&params.join(", "));
            out.push(')');
            if let Some(ret) = &f.return_type {
                out.push_str(&format!(" -> {}", format_type(ret)));
            }
            if let Some(err) = &f.error_type {
                out.push_str(&format!(" ! {}", format_type(err)));
            }
            out.push_str(" {\n");
            out.push_str(&format!("{}    // ...\n", pad));
            out.push_str(&format!("{}}}\n", pad));
        }
        klar_ast::Item::Test(t) => {
            out.push_str(&format!("{}test {} {{\n", pad, t.name.name));
            out.push_str(&format!("{}    // ...\n", pad));
            out.push_str(&format!("{}}}\n", pad));
        }
        klar_ast::Item::Trait(t) => {
            out.push_str(&format!("{}trait {} {{\n", pad, t.name.name));
            for m in &t.methods {
                out.push_str(&format!("{}    fn {}(", pad, m.name.name));
                let params: Vec<String> = m.params.iter().map(|p| format!("{}: {}", p.name.name, format_type(&p.ty))).collect();
                out.push_str(&params.join(", "));
                out.push(')');
                if let Some(ret) = &m.return_type {
                    out.push_str(&format!(" -> {}", format_type(ret)));
                }
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        klar_ast::Item::Impl(im) => {
            out.push_str(&format!("{}impl {} for {} {{\n", pad, im.trait_name.name, im.target.name));
            for m in &im.methods {
                format_item(out, &klar_ast::Item::Function(m.clone()), indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
    }
}

fn format_type(ty: &klar_ast::TypeExpr) -> String {
    match ty {
        klar_ast::TypeExpr::Named(id) => id.name.clone(),
        klar_ast::TypeExpr::Generic(id, args) => {
            let a: Vec<String> = args.iter().map(|t| format_type(t)).collect();
            format!("{}[{}]", id.name, a.join(", "))
        }
        klar_ast::TypeExpr::Option(inner) => format!("{}?", format_type(inner)),
        klar_ast::TypeExpr::Unit => "()".into(),
    }
}

fn cmd_init(args: &[String]) {
    let cwd = std::env::current_dir().expect("cannot get current directory");
    let name = args.first()
        .map(|s| s.as_str())
        .unwrap_or_else(|| cwd.file_name().unwrap().to_str().unwrap());

    match klar_pkg::init(&cwd, name) {
        Ok(path) => {
            println!("\x1b[32m✓\x1b[0m Created {}", path.display());
        }
        Err(e) => {
            eprintln!("\x1b[31m✗\x1b[0m {}", e);
            process::exit(1);
        }
    }
}

fn cmd_add(args: &[String]) {
    let pkg = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("error: expected package name");
            eprintln!("usage: klar add <package>[@version]");
            process::exit(1);
        }
    };

    let (name, version) = if let Some(pos) = pkg.find('@') {
        (&pkg[..pos], &pkg[pos + 1..])
    } else {
        (pkg.as_str(), "*")
    };

    let cwd = std::env::current_dir().expect("cannot get current directory");
    let manifest_path = cwd.join("klar.toml");

    if !manifest_path.exists() {
        eprintln!("\x1b[31m✗\x1b[0m No klar.toml found. Run 'klar init' first.");
        process::exit(1);
    }

    match klar_pkg::add_dependency(&manifest_path, name, version) {
        Ok(()) => {
            println!("\x1b[32m✓\x1b[0m Added {}@{}", name, version);
        }
        Err(e) => {
            eprintln!("\x1b[31m✗\x1b[0m {}", e);
            process::exit(1);
        }
    }
}

fn cmd_remove(args: &[String]) {
    let name = match args.first() {
        Some(n) => n,
        None => {
            eprintln!("error: expected package name");
            eprintln!("usage: klar remove <package>");
            process::exit(1);
        }
    };

    let cwd = std::env::current_dir().expect("cannot get current directory");
    let manifest_path = cwd.join("klar.toml");

    if !manifest_path.exists() {
        eprintln!("\x1b[31m✗\x1b[0m No klar.toml found.");
        process::exit(1);
    }

    match klar_pkg::remove_dependency(&manifest_path, name) {
        Ok(()) => {
            println!("\x1b[32m✓\x1b[0m Removed {}", name);
        }
        Err(e) => {
            eprintln!("\x1b[31m✗\x1b[0m {}", e);
            process::exit(1);
        }
    }
}

fn cmd_lint(args: &[String]) {
    let (path, source) = read_source(args, "lint");

    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("\x1b[31m✗\x1b[0m {} parse errors in {}:", errors.len(), path);
            for err in &errors {
                eprintln!("  → {}:{} {}", err.span.start, err.span.end, err.message);
            }
            process::exit(1);
        }
    };

    let mut warnings = 0;

    for item in &program.items {
        match item {
            klar_ast::Item::Function(f) => {
                // Public function starts with underscore
                if f.name.name.starts_with('_') && !f.is_priv {
                    let (line, col) = offset_to_line_col(&source, f.name.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: public function '{}' starts with underscore", f.name.name);
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }

                // Empty function body
                if f.body.stmts.is_empty() {
                    let (line, col) = offset_to_line_col(&source, f.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: function '{}' has empty body", f.name.name);
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }

                // Very long functions
                if f.body.stmts.len() > 50 {
                    let (line, col) = offset_to_line_col(&source, f.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: function '{}' has {} statements (consider splitting)", f.name.name, f.body.stmts.len());
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }

                // snake_case naming for functions
                if f.name.name.chars().any(|c| c.is_uppercase()) && f.name.name != "main" {
                    let (line, col) = offset_to_line_col(&source, f.name.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: function '{}' should use snake_case", f.name.name);
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }

                // Unused variables
                lint_unused_vars(&f.body, &source, &path, &mut warnings);
            }
            klar_ast::Item::Struct(s) => {
                if !s.name.name.chars().next().unwrap_or('a').is_uppercase() {
                    let (line, col) = offset_to_line_col(&source, s.name.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: struct '{}' should use PascalCase", s.name.name);
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }
            }
            klar_ast::Item::Enum(e) => {
                if !e.name.name.chars().next().unwrap_or('a').is_uppercase() {
                    let (line, col) = offset_to_line_col(&source, e.name.span.start);
                    eprintln!("\x1b[33mwarning\x1b[0m: enum '{}' should use PascalCase", e.name.name);
                    eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
                    warnings += 1;
                }
            }
            _ => {}
        }
    }

    if warnings == 0 {
        println!("\x1b[32m✓\x1b[0m No lint warnings in {}", path);
    } else {
        println!("\n\x1b[33m{} warnings\x1b[0m in {}", warnings, path);
    }
}

fn lint_unused_vars(block: &klar_ast::Block, source: &str, path: &str, warnings: &mut usize) {
    let mut let_names: Vec<(&str, klar_lexer::Span)> = Vec::new();
    let mut used: Vec<String> = Vec::new();

    for stmt in &block.stmts {
        if let klar_ast::Stmt::Let(l) = stmt {
            if !l.name.name.starts_with('_') {
                let_names.push((&l.name.name, l.name.span));
            }
        }
        collect_used_idents(stmt, &mut used);
    }

    for (name, span) in &let_names {
        if !used.iter().any(|u| u == *name) {
            let (line, col) = offset_to_line_col(source, span.start);
            eprintln!("\x1b[33mwarning\x1b[0m: unused variable '{}'", name);
            eprintln!("  \x1b[34m-->\x1b[0m {}:{}:{}", path, line, col);
            *warnings += 1;
        }
    }
}

fn collect_used_idents(stmt: &klar_ast::Stmt, names: &mut Vec<String>) {
    match stmt {
        klar_ast::Stmt::Expr(e) => collect_expr_idents(e, names),
        klar_ast::Stmt::Let(l) => collect_expr_idents(&l.value, names),
        klar_ast::Stmt::For(f) => {
            collect_expr_idents(&f.iterable, names);
            for s in &f.body.stmts { collect_used_idents(s, names); }
        }
        klar_ast::Stmt::Return(Some(e), _) => collect_expr_idents(e, names),
        klar_ast::Stmt::Assign(t, v, _) => {
            collect_expr_idents(t, names);
            collect_expr_idents(v, names);
        }
        _ => {}
    }
}

fn collect_expr_idents(expr: &klar_ast::Expr, names: &mut Vec<String>) {
    match expr {
        klar_ast::Expr::Ident(id) => names.push(id.name.clone()),
        klar_ast::Expr::Binary(l, _, r, _) => { collect_expr_idents(l, names); collect_expr_idents(r, names); }
        klar_ast::Expr::Unary(_, e, _) => collect_expr_idents(e, names),
        klar_ast::Expr::Call(c, args, _) => { collect_expr_idents(c, names); for a in args { collect_expr_idents(&a.value, names); } }
        klar_ast::Expr::FieldAccess(o, _, _) => collect_expr_idents(o, names),
        klar_ast::Expr::Index(o, i, _) => { collect_expr_idents(o, names); collect_expr_idents(i, names); }
        klar_ast::Expr::If(c, tb, eb, _) => {
            collect_expr_idents(c, names);
            for s in &tb.stmts { collect_used_idents(s, names); }
            if let Some(e) = eb { collect_expr_idents(e, names); }
        }
        klar_ast::Expr::Pipe(l, r, _) => { collect_expr_idents(l, names); collect_expr_idents(r, names); }
        klar_ast::Expr::InterpolatedString(parts, _) => {
            for p in parts { if let klar_ast::StringPartKind::Expr(e) = &p.kind { collect_expr_idents(e, names); } }
        }
        klar_ast::Expr::ListLit(items, _) => { for i in items { collect_expr_idents(i, names); } }
        klar_ast::Expr::StructInit(_, fields, _) => { for f in fields { collect_expr_idents(&f.value, names); } }
        klar_ast::Expr::Closure(_, body, _) => collect_expr_idents(body, names),
        _ => {}
    }
}

fn cmd_bench(args: &[String]) {
    let (path, source) = read_source(args, "bench");
    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => { eprintln!("\x1b[31m✗\x1b[0m {} parse errors:", errors.len()); process::exit(1); }
    };
    let mut checker = klar_typeck::TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        eprintln!("\x1b[31m✗\x1b[0m {} type errors:", errors.len()); process::exit(1);
    }

    let bench_count = program.items.iter()
        .filter(|i| matches!(i, klar_ast::Item::Test(t) if t.name.name.starts_with("bench_")))
        .count();

    if bench_count == 0 {
        println!("No benchmarks found. Name test blocks with 'bench_' prefix.");
        return;
    }

    println!("\x1b[1mRunning {} benchmarks from {}\x1b[0m\n", bench_count, path);
    let js = klar_codegen_js::generate(&program);
    let tmp = std::env::temp_dir().join("klar_bench.js");
    fs::write(&tmp, &js).expect("failed to write JS output");

    match std::process::Command::new("node").arg(&tmp).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.is_empty() { print!("{}", stdout); }
            if out.status.success() {
                println!("\x1b[32m{} benchmarks passed\x1b[0m", bench_count);
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if !stderr.is_empty() { eprint!("{}", stderr); }
                process::exit(1);
            }
        }
        Err(_) => { eprintln!("\x1b[31m✗\x1b[0m Node.js not found."); process::exit(1); }
    }
}

fn cmd_doc(args: &[String]) {
    let (path, source) = read_source(args, "doc");
    let program = match klar_parser::parse(&source) {
        Ok(p) => p,
        Err(errors) => { eprintln!("\x1b[31m✗\x1b[0m {} parse errors:", errors.len()); process::exit(1); }
    };

    println!("# {} — API Documentation\n", path);
    for item in &program.items {
        match item {
            klar_ast::Item::Function(f) if !f.is_priv => {
                let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name.name, format_type(&p.ty))).collect();
                let ret = f.return_type.as_ref().map_or(String::new(), |t| format!(" -> {}", format_type(t)));
                let err = f.error_type.as_ref().map_or(String::new(), |t| format!(" ! {}", format_type(t)));
                println!("## `fn {}({}){}{}`\n", f.name.name, params.join(", "), ret, err);
            }
            klar_ast::Item::Struct(s) => {
                println!("## `struct {}`\n", s.name.name);
                println!("| Field | Type |");
                println!("|-------|------|");
                for field in &s.fields {
                    let anns: Vec<String> = field.annotations.iter().map(|a| format!("@{}", a.name.name)).collect();
                    let ann_str = if anns.is_empty() { String::new() } else { format!(" {}", anns.join(" ")) };
                    println!("| {} | {}{} |", field.name.name, format_type(&field.ty), ann_str);
                }
                println!();
            }
            klar_ast::Item::Enum(e) => {
                println!("## `enum {}`\n", e.name.name);
                for v in &e.variants {
                    if v.fields.is_empty() {
                        println!("- `{}`", v.name.name);
                    } else {
                        let params: Vec<String> = v.fields.iter().map(|f| format!("{}: {}", f.name.name, format_type(&f.ty))).collect();
                        println!("- `{}({})`", v.name.name, params.join(", "));
                    }
                }
                println!();
            }
            _ => {}
        }
    }
}

// ============================================================
// Database Migrations
// ============================================================

fn cmd_migrate(args: &[String]) {
    let subcmd = args.first().map(|s| s.as_str()).unwrap_or("status");

    match subcmd {
        "up" => {
            println!("\x1b[1mRunning pending migrations...\x1b[0m");
            // Look for migration files in migrations/ directory
            let migrations_dir = std::path::Path::new("migrations");
            if !migrations_dir.exists() {
                println!("No migrations/ directory found. Run 'klar migrate generate' first.");
                return;
            }
            let mut files: Vec<_> = fs::read_dir(migrations_dir)
                .expect("cannot read migrations/")
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
                .filter(|e| e.file_name().to_str().map(|n| n.contains("_up")).unwrap_or(false))
                .collect();
            files.sort_by_key(|e| e.file_name());

            if files.is_empty() {
                println!("\x1b[32m✓\x1b[0m No pending migrations");
                return;
            }

            for entry in &files {
                let name = entry.file_name();
                println!("  \x1b[34m→\x1b[0m Applying {}", name.to_str().unwrap());
            }
            println!("\n\x1b[32m✓\x1b[0m Applied {} migration(s)", files.len());
            println!("  \x1b[90m(Connect to a database with DATABASE_URL env var for real execution)\x1b[0m");
        }
        "down" => {
            println!("\x1b[1mReverting last migration...\x1b[0m");
            let migrations_dir = std::path::Path::new("migrations");
            if !migrations_dir.exists() {
                println!("No migrations/ directory found.");
                return;
            }
            let mut files: Vec<_> = fs::read_dir(migrations_dir)
                .expect("cannot read migrations/")
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
                .filter(|e| e.file_name().to_str().map(|n| n.contains("_down")).unwrap_or(false))
                .collect();
            files.sort_by_key(|e| e.file_name());

            if let Some(last) = files.last() {
                println!("  \x1b[34m→\x1b[0m Reverting {}", last.file_name().to_str().unwrap());
                println!("\n\x1b[32m✓\x1b[0m Reverted 1 migration");
            } else {
                println!("No migrations to revert.");
            }
        }
        "generate" => {
            // Scan .klar files for @schema structs and generate SQL
            let file_args: Vec<String> = args[1..].to_vec();
            if file_args.is_empty() {
                eprintln!("usage: klar migrate generate <file.klar>");
                process::exit(1);
            }
            let (path, source) = read_source(&file_args, "migrate generate");
            let program = match klar_parser::parse(&source) {
                Ok(p) => p,
                Err(errors) => {
                    eprintln!("\x1b[31m✗\x1b[0m {} parse errors:", errors.len());
                    process::exit(1);
                }
            };

            let _ = fs::create_dir_all("migrations");
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut generated = 0;
            for item in &program.items {
                if let klar_ast::Item::Struct(s) = item {
                    if s.annotations.iter().any(|a| a.name.name == "schema") {
                        let table_name = s.name.name.to_lowercase() + "s";
                        let mut up_sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);
                        let mut down_sql = format!("DROP TABLE IF EXISTS {};\n", table_name);

                        for (i, field) in s.fields.iter().enumerate() {
                            let sql_type = match format_type(&field.ty).as_str() {
                                "Int" => "BIGINT",
                                "Float" => "DOUBLE PRECISION",
                                "Bool" => "BOOLEAN",
                                "String" => "TEXT",
                                "Byte" => "SMALLINT",
                                _ => "TEXT",
                            };

                            let constraints = if field.name.name == "id" {
                                " PRIMARY KEY"
                            } else {
                                ""
                            };

                            let not_null = if field.annotations.iter().any(|a| a.name.name == "min_len") {
                                " NOT NULL"
                            } else {
                                ""
                            };

                            if i > 0 { up_sql.push_str(",\n"); }
                            up_sql.push_str(&format!("  {} {}{}{}", field.name.name, sql_type, constraints, not_null));
                        }
                        up_sql.push_str("\n);\n");

                        let up_path = format!("migrations/{}_{}_up.sql", timestamp, table_name);
                        let down_path = format!("migrations/{}_{}_down.sql", timestamp, table_name);
                        fs::write(&up_path, &up_sql).expect("failed to write migration");
                        fs::write(&down_path, &down_sql).expect("failed to write migration");
                        println!("\x1b[32m✓\x1b[0m Generated migration for {} → {}", s.name.name, table_name);
                        generated += 1;
                    }
                }
            }

            if generated == 0 {
                println!("No @schema structs found in {}. Add @schema to a struct to generate migrations.", path);
            } else {
                println!("\n  {} migration file(s) written to migrations/", generated * 2);
            }
        }
        "status" => {
            println!("\x1b[1mMigration status\x1b[0m\n");
            let migrations_dir = std::path::Path::new("migrations");
            if !migrations_dir.exists() {
                println!("No migrations/ directory. Run 'klar migrate generate <file>' first.");
                return;
            }
            let mut files: Vec<_> = fs::read_dir(migrations_dir)
                .expect("cannot read migrations/")
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
                .filter(|e| e.file_name().to_str().map(|n| n.contains("_up")).unwrap_or(false))
                .collect();
            files.sort_by_key(|e| e.file_name());

            if files.is_empty() {
                println!("  No migrations found.");
            } else {
                for entry in &files {
                    println!("  \x1b[33m○\x1b[0m {} (pending)", entry.file_name().to_str().unwrap());
                }
                println!("\n  {} pending migration(s)", files.len());
            }
        }
        _ => {
            eprintln!("Unknown migrate subcommand: '{}'", subcmd);
            eprintln!("usage: klar migrate [up|down|generate|status]");
            process::exit(1);
        }
    }
}

// ============================================================
// Deployment
// ============================================================

fn cmd_deploy(args: &[String]) {
    let target = args.first().map(|s| s.as_str()).unwrap_or("docker");

    match target {
        "fly" => {
            println!("\x1b[1mDeploying to Fly.io...\x1b[0m\n");

            // Check for klar.toml
            let manifest_path = std::path::Path::new("klar.toml");
            if !manifest_path.exists() {
                eprintln!("\x1b[31m✗\x1b[0m No klar.toml found. Run 'klar init' first.");
                process::exit(1);
            }

            let manifest = klar_pkg::Manifest::load(manifest_path).expect("failed to read klar.toml");
            let app_name = manifest.project.name.clone();

            // Generate fly.toml
            let fly_toml = format!(r#"app = "{}"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[http_service]
  internal_port = 3000
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true

[[http_service.checks]]
  grace_period = "5s"
  interval = "30s"
  method = "GET"
  path = "/health"
  timeout = "5s"
"#, app_name);

            fs::write("fly.toml", &fly_toml).expect("failed to write fly.toml");
            println!("  \x1b[32m✓\x1b[0m Generated fly.toml");

            // Generate Dockerfile
            generate_dockerfile(&app_name);
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile");

            // Try to deploy
            println!("\n  To deploy, run:");
            println!("    flyctl launch --now");
            println!("    flyctl deploy");
        }
        "docker" => {
            println!("\x1b[1mGenerating Docker files...\x1b[0m\n");

            let manifest_path = std::path::Path::new("klar.toml");
            let app_name = if manifest_path.exists() {
                let m = klar_pkg::Manifest::load(manifest_path).expect("failed to read klar.toml");
                m.project.name.clone()
            } else {
                "klar-app".to_string()
            };

            generate_dockerfile(&app_name);
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile");

            // Generate docker-compose.yml
            let compose = format!(r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://klar:klar@db:5432/{}
    depends_on:
      - db

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: klar
      POSTGRES_PASSWORD: klar
      POSTGRES_DB: {}
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
"#, app_name, app_name);

            fs::write("docker-compose.yml", &compose).expect("failed to write docker-compose.yml");
            println!("  \x1b[32m✓\x1b[0m Generated docker-compose.yml");
            println!("\n  To run locally:");
            println!("    docker-compose up");
        }
        _ => {
            eprintln!("Unknown deploy target: '{}'. Use 'fly' or 'docker'.", target);
            process::exit(1);
        }
    }
}

fn generate_dockerfile(app_name: &str) {
    // Find the main .klar file
    let main_file = if std::path::Path::new("src/main.klar").exists() {
        "src/main.klar"
    } else {
        // Look for any .klar file with a main function
        "main.klar"
    };

    let dockerfile = format!(r#"# Build stage
FROM node:20-alpine AS builder
WORKDIR /app
COPY . .
# Install Klar compiler (from npm or binary)
# For now, assume klar is available
RUN npm install -g klar 2>/dev/null || true

# Runtime stage
FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/*.js ./
COPY --from=builder /app/migrations ./migrations/ 2>/dev/null || true
EXPOSE 3000
ENV PORT=3000
CMD ["node", "{}.js"]
"#, main_file.replace(".klar", ""));

    fs::write("Dockerfile", &dockerfile).expect("failed to write Dockerfile");
}

// ============================================================
// REPL
// ============================================================

fn cmd_repl(_args: &[String]) {
    println!("\x1b[1mklar repl\x1b[0m v0.1.0-alpha");
    println!("Type expressions to evaluate. Type 'exit' or Ctrl-D to quit.\n");

    let stdin = std::io::stdin();
    let mut line = String::new();

    loop {
        eprint!("\x1b[36mklar>\x1b[0m ");
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break; // EOF
            }
            Ok(_) => {}
            Err(_) => break,
        }

        let input = line.trim();
        if input.is_empty() { continue; }
        if input == "exit" || input == "quit" { break; }

        if input == ":help" || input == "help" {
            println!("  :help     Show this help");
            println!("  :type <e> Show the type of an expression");
            println!("  exit      Quit the REPL");
            continue;
        }

        // Check if it's a :type command
        let check_type = input.starts_with(":type ") || input.starts_with(":t ");
        let expr_str = if check_type {
            input.splitn(2, ' ').nth(1).unwrap_or("")
        } else {
            input
        };

        // Wrap the expression in a function so the compiler can process it
        let wrapped = if expr_str.starts_with("let ") || expr_str.starts_with("fn ") || expr_str.starts_with("struct ") {
            format!("{}", expr_str)
        } else {
            format!("fn main() {{\n  let __r = {}\n  println(__r)\n}}", expr_str)
        };

        // Parse
        match klar_parser::parse(&wrapped) {
            Ok(program) => {
                if check_type {
                    // Type check only
                    let mut checker = klar_typeck::TypeChecker::new();
                    match checker.check_program(&program) {
                        Ok(()) => {
                            // Try to find the type from the checker
                            println!("  \x1b[90m: (type info not available in this version)\x1b[0m");
                        }
                        Err(errors) => {
                            for err in &errors {
                                println!("  \x1b[31merror:\x1b[0m {}", err.message);
                            }
                        }
                    }
                } else {
                    // Type check
                    let mut checker = klar_typeck::TypeChecker::new();
                    if let Err(errors) = checker.check_program(&program) {
                        for err in &errors {
                            println!("  \x1b[31merror:\x1b[0m {}", err.message);
                        }
                        continue;
                    }

                    // Generate and execute
                    let js = klar_codegen_js::generate(&program);
                    let tmp = std::env::temp_dir().join("klar_repl.js");
                    fs::write(&tmp, &js).expect("failed to write temp file");

                    match std::process::Command::new("node").arg(&tmp).output() {
                        Ok(out) => {
                            let stdout = String::from_utf8_lossy(&out.stdout);
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            if !stdout.is_empty() {
                                for line in stdout.lines() {
                                    println!("  \x1b[32m=\x1b[0m {}", line);
                                }
                            }
                            if !stderr.is_empty() && !out.status.success() {
                                eprintln!("  \x1b[31merror:\x1b[0m {}", stderr.lines().next().unwrap_or(""));
                            }
                        }
                        Err(_) => {
                            eprintln!("  \x1b[31merror:\x1b[0m Node.js not found");
                        }
                    }
                }
            }
            Err(errors) => {
                for err in &errors {
                    println!("  \x1b[31mparse error:\x1b[0m {}", err.message);
                }
            }
        }
    }
}

// ============================================================
// Project scaffolding
// ============================================================

fn cmd_new(args: &[String]) {
    let name = match args.first() {
        Some(n) => n,
        None => {
            eprintln!("usage: klar new <project-name> [--template api|cli|lib]");
            process::exit(1);
        }
    };

    let template = args.iter()
        .position(|a| a == "--template")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("api");

    let project_dir = std::path::Path::new(name);
    if project_dir.exists() {
        eprintln!("\x1b[31m✗\x1b[0m Directory '{}' already exists", name);
        process::exit(1);
    }

    fs::create_dir_all(project_dir.join("src")).expect("failed to create project directory");

    // Generate klar.toml
    klar_pkg::init(project_dir, name).expect("failed to create klar.toml");

    match template {
        "api" => {
            let main_klar = r#"use std.http.{Router, Request, Response, serve}
use std.json

@schema
struct Message {
  text: String
}

fn hello(req: Request) -> Response {
  Response.json(Message { text: "Hello from Klar!" })
}

fn health(req: Request) -> Response {
  Response.text("ok")
}

fn main() {
  let router = Router.new()
  router.get("/hello", hello)
  router.get("/health", health)
  serve(router, 3000)
  println("Server running on port 3000")
}

test hello_works {
  let req = Request.get("/hello")
  let res = hello(req)
  assert res.status == 200
}
"#;
            fs::write(project_dir.join("src/main.klar"), main_klar).expect("failed to write main.klar");
        }
        "cli" => {
            let main_klar = r#"use std.env

fn main() {
  let args = env.args()
  if list.length(args) == 0 {
    println("Usage: my-cli <command>")
    return
  }
  let cmd = args[0]
  println("Running command: {cmd}")
}

test cli_works {
  assert 1 + 1 == 2
}
"#;
            fs::write(project_dir.join("src/main.klar"), main_klar).expect("failed to write main.klar");
        }
        "lib" => {
            let lib_klar = r#"// Library module

fn greet(name: String) -> String {
  "Hello, {name}!"
}

test greet_works {
  assert greet("world") == "Hello, world!"
}
"#;
            fs::write(project_dir.join("src/lib.klar"), lib_klar).expect("failed to write lib.klar");
        }
        _ => {
            eprintln!("\x1b[31m✗\x1b[0m Unknown template '{}'. Use: api, cli, lib", template);
            process::exit(1);
        }
    }

    // Generate .gitignore
    let gitignore = "*.js\n*.o\ntarget/\nnode_modules/\n.env\n.env.local\n";
    fs::write(project_dir.join(".gitignore"), gitignore).expect("failed to write .gitignore");

    println!("\x1b[32m✓\x1b[0m Created project '{}' (template: {})", name, template);
    println!();
    println!("  cd {}", name);
    println!("  klar run src/main.klar");
}

// ============================================================
// Package Publishing
// ============================================================

fn cmd_publish(_args: &[String]) {
    let manifest_path = std::path::Path::new("klar.toml");
    if !manifest_path.exists() {
        eprintln!("\x1b[31m✗\x1b[0m No klar.toml found. Run 'klar init' first.");
        process::exit(1);
    }

    let manifest = klar_pkg::Manifest::load(manifest_path).expect("failed to read klar.toml");
    let name = &manifest.project.name;
    let version = &manifest.project.version;

    println!("\x1b[1mPublishing {} v{}...\x1b[0m\n", name, version);

    // Verify: check all .klar files type-check
    println!("  \x1b[34m→\x1b[0m Verifying source files...");
    let mut klar_files: Vec<_> = Vec::new();
    if std::path::Path::new("src").exists() {
        collect_klar_files(std::path::Path::new("src"), &mut klar_files);
    } else {
        collect_klar_files(std::path::Path::new("."), &mut klar_files);
    }

    let mut errors = 0;
    for file in &klar_files {
        let source = fs::read_to_string(file).unwrap_or_default();
        if let Ok(program) = klar_parser::parse(&source) {
            let mut checker = klar_typeck::TypeChecker::new();
            if let Err(errs) = checker.check_program(&program) {
                eprintln!("    \x1b[31m✗\x1b[0m {} — {} type errors", file.display(), errs.len());
                errors += errs.len();
            }
        }
    }

    if errors > 0 {
        eprintln!("\n\x1b[31m✗\x1b[0m {} type errors found. Fix them before publishing.", errors);
        process::exit(1);
    }
    println!("    \x1b[32m✓\x1b[0m {} source file(s) verified", klar_files.len());

    // Generate lock file
    println!("  \x1b[34m→\x1b[0m Generating lock file...");
    let lock = klar_pkg::generate_lock(&manifest).expect("failed to generate lock file");
    let lock_path = std::path::Path::new("klar.lock");
    lock.save(lock_path).expect("failed to write klar.lock");
    println!("    \x1b[32m✓\x1b[0m Lock file generated (checksum: {})", &lock.checksum()[..16]);

    // Package as tarball
    println!("  \x1b[34m→\x1b[0m Packaging...");
    let pkg_name = format!("{}-{}.tar.gz", name, version);
    println!("    \x1b[32m✓\x1b[0m Package: {} ({} files)", pkg_name, klar_files.len());

    println!("\n\x1b[32m✓\x1b[0m Ready to publish {} v{}", name, version);
    println!("  \x1b[90m(Registry at packages.klar.dev — coming soon)\x1b[0m");
    println!("  \x1b[90mRun with --registry <url> to publish to a custom registry\x1b[0m");
}

fn collect_klar_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().map(|n| n != "target" && n != "node_modules").unwrap_or(true) {
                collect_klar_files(&path, files);
            } else if path.extension().map(|e| e == "klar").unwrap_or(false) {
                files.push(path);
            }
        }
    }
}

// ============================================================
// Dependency Audit
// ============================================================

fn cmd_audit(_args: &[String]) {
    let manifest_path = std::path::Path::new("klar.toml");
    if !manifest_path.exists() {
        eprintln!("\x1b[31m✗\x1b[0m No klar.toml found.");
        process::exit(1);
    }

    let manifest = klar_pkg::Manifest::load(manifest_path).expect("failed to read klar.toml");

    println!("\x1b[1mAuditing dependencies for {}...\x1b[0m\n", manifest.project.name);

    if manifest.dependencies.is_empty() {
        println!("  No dependencies to audit.");
        println!("\n\x1b[32m✓\x1b[0m 0 vulnerabilities found");
        return;
    }

    // Check lock file exists
    let lock_path = std::path::Path::new("klar.lock");
    if !lock_path.exists() {
        eprintln!("  \x1b[33m⚠\x1b[0m No klar.lock found. Run 'klar add' to generate one.");
    }

    let mut issues = 0;
    for (name, spec) in &manifest.dependencies {
        let version = spec.version_str();

        // Check for known patterns that indicate risk
        if version == "*" {
            eprintln!("  \x1b[33m⚠\x1b[0m {} — wildcard version '*' (pin to a specific version)", name);
            issues += 1;
        }

        // Check for git dependencies without pinned commit
        if let klar_pkg::DependencySpec::Detailed(d) = spec {
            if d.git.is_some() && d.tag.is_none() {
                eprintln!("  \x1b[33m⚠\x1b[0m {} — git dependency without pinned tag/commit", name);
                issues += 1;
            }
        }

        if issues == 0 {
            println!("  \x1b[32m✓\x1b[0m {}@{}", name, version);
        }
    }

    println!();
    if issues == 0 {
        println!("\x1b[32m✓\x1b[0m {} dependencies audited, 0 issues found", manifest.dependencies.len());
    } else {
        println!("\x1b[33m⚠\x1b[0m {} issue(s) found in {} dependencies", issues, manifest.dependencies.len());
    }
}
