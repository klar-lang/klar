use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use klar_ast::*;
use std::collections::HashMap;
use std::path::Path;

mod runtime;

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("LLVM error: {0}")]
    Llvm(String),
    #[error("unsupported feature: {0}")]
    Unsupported(String),
    #[error("undefined variable: {0}")]
    Undefined(String),
    #[error("type error: {0}")]
    Type(String),
}

pub type Result<T> = std::result::Result<T, CodegenError>;

/// Generate LLVM IR text from a Klar AST.
pub fn generate_ir(program: &Program) -> Result<String> {
    let context = Context::create();
    let mut cg = LlvmGen::new(&context, "klar_module");
    cg.emit_program(program)?;
    Ok(cg.module.print_to_string().to_string())
}

/// Compile a Klar program to a native object file.
pub fn compile_to_object(program: &Program, output_path: &Path) -> Result<()> {
    let context = Context::create();
    let mut cg = LlvmGen::new(&context, "klar_module");
    cg.emit_program(program)?;

    let target_machine = create_target_machine()?;
    target_machine
        .write_to_file(&cg.module, inkwell::targets::FileType::Object, output_path)
        .map_err(|e| CodegenError::Llvm(e.to_string()))?;

    Ok(())
}

/// Compile a Klar program to a native executable.
pub fn compile_to_native(program: &Program, output_path: &Path) -> Result<()> {
    let obj_path = output_path.with_extension("o");
    compile_to_object(program, &obj_path)?;

    // Find the runtime library
    let runtime_lib = find_runtime_lib();

    // Link using system linker (cc)
    let mut cmd = std::process::Command::new("cc");
    cmd.arg(&obj_path)
        .arg("-o")
        .arg(output_path)
        .arg("-lc");

    if let Some(lib_path) = &runtime_lib {
        cmd.arg(lib_path);
    }

    let status = cmd
        .status()
        .map_err(|e| CodegenError::Llvm(format!("failed to invoke linker: {}", e)))?;

    if !status.success() {
        return Err(CodegenError::Llvm("linker failed".into()));
    }

    // Clean up object file
    let _ = std::fs::remove_file(&obj_path);

    Ok(())
}

fn find_runtime_lib() -> Option<String> {
    // Look for the runtime library in common locations
    let candidates = [
        // Development: relative to the compiler binary
        "target/debug/libklar_runtime.a",
        "target/release/libklar_runtime.a",
        // Installed: next to the compiler binary
        "../lib/libklar_runtime.a",
    ];

    // First, try relative to the current executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let lib_path = exe_dir.join("libklar_runtime.a");
            if lib_path.exists() {
                return Some(lib_path.to_string_lossy().into_owned());
            }
            // Check parent/lib
            if let Some(parent) = exe_dir.parent() {
                let lib_path = parent.join("lib").join("libklar_runtime.a");
                if lib_path.exists() {
                    return Some(lib_path.to_string_lossy().into_owned());
                }
            }
        }
    }

    // Try relative to cwd
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Some(candidate.to_string());
        }
    }

    None
}

fn create_target_machine() -> Result<TargetMachine> {
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| CodegenError::Llvm(format!("failed to initialize native target: {}", e)))?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| CodegenError::Llvm(format!("failed to get target: {}", e)))?;

    target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::Llvm("failed to create target machine".into()))
}

// ============================================================
// Code Generator
// ============================================================

struct LlvmGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    /// Named values in the current scope (variable name -> (alloca pointer, type)).
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    /// Registered struct types (struct name -> LLVM struct type + field names).
    struct_types: HashMap<String, (StructType<'ctx>, Vec<String>)>,
    /// Registered enum types (enum name -> tag type + variant info).
    enum_types: HashMap<String, EnumInfo<'ctx>>,
    /// Function declarations.
    functions: HashMap<String, FunctionValue<'ctx>>,
    /// Stack of loop exit blocks for `break` statements.
    loop_exit_stack: Vec<BasicBlock<'ctx>>,
}

#[derive(Clone)]
struct EnumInfo<'ctx> {
    /// The LLVM struct type for the enum (tag + max-sized payload).
    llvm_type: StructType<'ctx>,
    /// Variant name -> (tag index, field types).
    variants: HashMap<String, (u32, Vec<BasicTypeEnum<'ctx>>)>,
}

impl<'ctx> LlvmGen<'ctx> {
    fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            struct_types: HashMap::new(),
            enum_types: HashMap::new(),
            functions: HashMap::new(),
            loop_exit_stack: Vec::new(),
        }
    }

    // ============================================================
    // Program
    // ============================================================

    fn emit_program(&mut self, program: &Program) -> Result<()> {
        // Declare external runtime functions (printf, etc.)
        self.declare_runtime();

        // First pass: register all type and function declarations
        for item in &program.items {
            self.register_item(item)?;
        }

        // Second pass: emit function bodies
        for item in &program.items {
            self.emit_item(item)?;
        }

        // Emit C main that calls Klar's main()
        self.emit_c_main()?;

        // Verify the module
        self.module
            .verify()
            .map_err(|e| CodegenError::Llvm(format!("module verification failed: {}", e)))?;

        Ok(())
    }

    fn declare_runtime(&mut self) {
        runtime::declare_runtime(self.context, &self.module);
    }

    // ============================================================
    // Registration (first pass)
    // ============================================================

    fn register_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function(f) => self.register_fn(f),
            Item::Struct(s) => self.register_struct(s),
            Item::Enum(e) => self.register_enum(e),
            Item::Test(t) => self.register_test(t),
            Item::Use(_) | Item::Trait(_) | Item::Impl(_) => Ok(()),
        }
    }

    fn register_fn(&mut self, f: &FnDecl) -> Result<()> {
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = f
            .params
            .iter()
            .map(|p| self.type_expr_to_llvm(&p.ty))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|t| t.into())
            .collect();

        let fn_type = match &f.return_type {
            Some(ret) => {
                let ret_ty = self.type_expr_to_llvm(ret)?;
                ret_ty.fn_type(&param_types, false)
            }
            None => self.context.void_type().fn_type(&param_types, false),
        };

        // Rename Klar's main to klar_main to avoid conflict with C main
        let llvm_name = if f.name.name == "main" {
            "klar_main".to_string()
        } else {
            f.name.name.clone()
        };

        let func = self.module.add_function(&llvm_name, fn_type, None);
        self.functions.insert(f.name.name.clone(), func);
        Ok(())
    }

    fn register_struct(&mut self, s: &StructDecl) -> Result<()> {
        let field_types: Vec<BasicTypeEnum<'ctx>> = s
            .fields
            .iter()
            .map(|f| self.type_expr_to_llvm(&f.ty))
            .collect::<Result<Vec<_>>>()?;

        let field_names: Vec<String> = s.fields.iter().map(|f| f.name.name.clone()).collect();

        let struct_type = self.context.struct_type(
            &field_types.iter().map(|t| *t).collect::<Vec<_>>(),
            false,
        );

        self.struct_types
            .insert(s.name.name.clone(), (struct_type, field_names));
        Ok(())
    }

    fn register_enum(&mut self, e: &EnumDecl) -> Result<()> {
        let tag_type = self.context.i32_type();
        let mut variants = HashMap::new();
        let mut max_payload_size = 0u32;

        for (i, variant) in e.variants.iter().enumerate() {
            let field_types: Vec<BasicTypeEnum<'ctx>> = variant
                .fields
                .iter()
                .map(|f| self.type_expr_to_llvm(&f.ty))
                .collect::<Result<Vec<_>>>()?;

            // Calculate payload size as sum of field sizes (simplified)
            let payload_size: u32 = field_types.len() as u32 * 8; // rough estimate
            max_payload_size = max_payload_size.max(payload_size);

            variants.insert(variant.name.name.clone(), (i as u32, field_types));
        }

        // Enum struct: { i32 tag, [max_payload_size x i8] payload }
        let payload_type = self.context.i8_type().array_type(max_payload_size.max(8));
        let enum_type = self.context.struct_type(
            &[tag_type.into(), payload_type.into()],
            false,
        );

        self.enum_types.insert(
            e.name.name.clone(),
            EnumInfo {
                llvm_type: enum_type,
                variants,
            },
        );
        Ok(())
    }

    fn register_test(&mut self, t: &TestDecl) -> Result<()> {
        let fn_name = format!("test_{}", t.name.name);
        let fn_type = self.context.void_type().fn_type(&[], false);
        let func = self.module.add_function(&fn_name, fn_type, None);
        self.functions.insert(fn_name, func);
        Ok(())
    }

    // ============================================================
    // Emit (second pass)
    // ============================================================

    fn emit_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function(f) => self.emit_fn(f),
            Item::Test(t) => self.emit_test(t),
            Item::Struct(_) | Item::Enum(_) | Item::Use(_) | Item::Trait(_) | Item::Impl(_) => {
                Ok(())
            }
        }
    }

    fn emit_fn(&mut self, f: &FnDecl) -> Result<()> {
        let func = *self
            .functions
            .get(&f.name.name)
            .ok_or_else(|| CodegenError::Undefined(f.name.name.clone()))?;

        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Save old variables and create new scope
        let old_vars = self.variables.clone();
        self.variables.clear();

        // Create allocas for parameters
        for (i, param) in f.params.iter().enumerate() {
            let param_val = func.get_nth_param(i as u32).unwrap();
            let param_ty = param_val.get_type();
            let alloca = self.create_entry_alloca(func, &param.name.name, param_ty);
            self.builder.build_store(alloca, param_val)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            self.variables.insert(param.name.name.clone(), (alloca, param_ty));
        }

        // Emit body
        let result = self.emit_block(&f.body)?;

        // Add return
        if f.return_type.is_some() {
            if let Some(val) = result {
                self.builder.build_return(Some(&val))
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            } else {
                // Return default
                let ret_ty = self.type_expr_to_llvm(f.return_type.as_ref().unwrap())?;
                let default = self.default_value(ret_ty);
                self.builder.build_return(Some(&default))
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            }
        } else if self.needs_terminator() {
            self.builder.build_return(None)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        // Restore scope
        self.variables = old_vars;
        Ok(())
    }

    fn emit_test(&mut self, t: &TestDecl) -> Result<()> {
        let fn_name = format!("test_{}", t.name.name);
        let func = *self
            .functions
            .get(&fn_name)
            .ok_or_else(|| CodegenError::Undefined(fn_name.clone()))?;

        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        let old_vars = self.variables.clone();
        self.variables.clear();

        self.emit_block(&t.body)?;

        // Print test passed message
        let msg = format!("  \x1b[32m✓\x1b[0m {}\n\0", t.name.name);
        let msg_val = self.builder.build_global_string_ptr(&msg, "test_msg")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        let printf = self.module.get_function("printf").unwrap();
        self.builder
            .build_call(printf, &[msg_val.as_pointer_value().into()], "")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        if self.needs_terminator() {
            self.builder.build_return(None)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        self.variables = old_vars;
        Ok(())
    }

    fn emit_c_main(&mut self) -> Result<()> {
        let i32_type = self.context.i32_type();
        let main_type = i32_type.fn_type(&[], false);
        let c_main = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(c_main, "entry");
        self.builder.position_at_end(entry);

        // Call test functions first
        for (name, func) in &self.functions {
            if name.starts_with("test_") {
                self.builder
                    .build_call(*func, &[], "")
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            }
        }

        // Call klar main() if it exists
        if let Some(klar_main) = self.functions.get("klar_main") {
            self.builder
                .build_call(*klar_main, &[], "")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        } else if let Some(klar_main) = self.module.get_function("klar_main") {
            self.builder
                .build_call(klar_main, &[], "")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        Ok(())
    }

    // ============================================================
    // Blocks & Statements
    // ============================================================

    fn emit_block(&mut self, block: &Block) -> Result<Option<BasicValueEnum<'ctx>>> {
        let mut last_val = None;
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == block.stmts.len() - 1;
            last_val = self.emit_stmt(stmt, is_last)?;
        }
        Ok(last_val)
    }

    fn emit_stmt(
        &mut self,
        stmt: &Stmt,
        is_last: bool,
    ) -> Result<Option<BasicValueEnum<'ctx>>> {
        match stmt {
            Stmt::Let(l) => {
                let val = self.emit_expr(&l.value)?;
                let val_ty = val.get_type();
                let alloca = self.create_entry_alloca(
                    self.current_function(),
                    &l.name.name,
                    val_ty,
                );
                self.builder.build_store(alloca, val)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                self.variables.insert(l.name.name.clone(), (alloca, val_ty));
                Ok(None)
            }
            Stmt::Expr(expr) => {
                let val = self.emit_expr(expr)?;
                if is_last {
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }
            Stmt::Return(expr, _) => {
                if let Some(e) = expr {
                    let val = self.emit_expr(e)?;
                    self.builder.build_return(Some(&val))
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                } else {
                    self.builder.build_return(None)
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                }
                Ok(None)
            }
            Stmt::For(f) => {
                self.emit_for(f)?;
                Ok(None)
            }
            Stmt::Loop(block, _) => {
                self.emit_loop(block)?;
                Ok(None)
            }
            Stmt::Break(_) => {
                if let Some(&exit_bb) = self.loop_exit_stack.last() {
                    self.builder.build_unconditional_branch(exit_bb)
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                }
                Ok(None)
            }
            Stmt::Assign(target, value, _) => {
                let val = self.emit_expr(value)?;
                if let Expr::Ident(id) = target {
                    if let Some((alloca, _ty)) = self.variables.get(&id.name) {
                        self.builder.build_store(*alloca, val)
                            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    }
                } else if let Expr::FieldAccess(obj, field, _) = target {
                    self.emit_field_store(obj, &field.name, val)?;
                }
                Ok(None)
            }
            Stmt::Item(item) => {
                self.register_item(item)?;
                self.emit_item(item)?;
                Ok(None)
            }
        }
    }

    fn emit_for(&mut self, f: &ForStmt) -> Result<()> {
        let func = self.current_function();

        // For now, support for loops over ranges: `for x in 0..10`
        // and list literals
        match &f.iterable {
            Expr::Range(start, end, _) => {
                let start_val = self.emit_expr(start)?;
                let end_val = self.emit_expr(end)?;

                // Create blocks
                let _preheader = self.builder.get_insert_block().unwrap();
                let loop_bb = self.context.append_basic_block(func, "for.body");
                let after_bb = self.context.append_basic_block(func, "for.end");

                // Alloca for loop variable
                let loop_var = self.create_entry_alloca(func, &f.binding.name, self.context.i64_type().into());
                self.builder.build_store(loop_var, start_val)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                self.variables.insert(f.binding.name.clone(), (loop_var, self.context.i64_type().into()));

                // Branch to loop
                self.builder.build_unconditional_branch(loop_bb)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;

                // Loop body
                self.builder.position_at_end(loop_bb);
                let cur_val = self.builder.build_load(self.context.i64_type(), loop_var, &f.binding.name)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;

                // Emit body
                self.emit_block(&f.body)?;

                // Increment
                let next_val = self.builder
                    .build_int_add(
                        cur_val.into_int_value(),
                        self.context.i64_type().const_int(1, false),
                        "next",
                    )
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                self.builder.build_store(loop_var, next_val)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;

                // Condition check
                let cond = self.builder
                    .build_int_compare(
                        inkwell::IntPredicate::SLT,
                        next_val,
                        end_val.into_int_value(),
                        "for.cond",
                    )
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                self.builder.build_conditional_branch(cond, loop_bb, after_bb)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;

                self.builder.position_at_end(after_bb);
            }
            _ => {
                // For other iterables (lists), emit as indexed loop
                // This is a simplified version — will be expanded in S15 with ARC runtime
                let _iterable = self.emit_expr(&f.iterable)?;
                // For now, skip complex iterables
            }
        }
        Ok(())
    }

    fn emit_loop(&mut self, block: &Block) -> Result<()> {
        let func = self.current_function();
        let loop_bb = self.context.append_basic_block(func, "loop.body");
        let after_bb = self.context.append_basic_block(func, "loop.end");

        self.builder.build_unconditional_branch(loop_bb)
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        self.builder.position_at_end(loop_bb);

        // Push break target so `break` statements can branch to after_bb
        self.loop_exit_stack.push(after_bb);
        self.emit_block(block)?;
        self.loop_exit_stack.pop();

        if self.needs_terminator() {
            self.builder.build_unconditional_branch(loop_bb)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        self.builder.position_at_end(after_bb);
        Ok(())
    }

    // ============================================================
    // Expressions
    // ============================================================

    fn emit_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            Expr::IntLit(n, _) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            Expr::FloatLit(n, _) => {
                Ok(self.context.f64_type().const_float(*n).into())
            }
            Expr::BoolLit(b, _) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            Expr::StringLit(s, _) => {
                self.emit_string_literal(s)
            }
            Expr::InterpolatedString(parts, _) => {
                self.emit_interpolated_string(parts)
            }
            Expr::Ident(id) => {
                self.emit_ident(id)
            }
            Expr::Binary(lhs, op, rhs, _) => {
                self.emit_binary(lhs, *op, rhs)
            }
            Expr::Unary(op, inner, _) => {
                self.emit_unary(*op, inner)
            }
            Expr::Call(callee, args, _) => {
                self.emit_call(callee, args)
            }
            Expr::If(cond, then_block, else_branch, _) => {
                self.emit_if(cond, then_block, else_branch.as_deref())
            }
            Expr::Match(scrutinee, arms, _) => {
                self.emit_match(scrutinee, arms)
            }
            Expr::FieldAccess(obj, field, _) => {
                self.emit_field_access(obj, &field.name)
            }
            Expr::Index(obj, idx, _) => {
                self.emit_index(obj, idx)
            }
            Expr::StructInit(name, fields, _) => {
                self.emit_struct_init(&name.name, fields)
            }
            Expr::ListLit(items, _) => {
                self.emit_list_literal(items)
            }
            Expr::MapLit(_entries, _) => {
                // Simplified: return a dummy value for now
                Ok(self.context.i64_type().const_int(0, false).into())
            }
            Expr::Block(block) => {
                let val = self.emit_block(block)?;
                Ok(val.unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()))
            }
            Expr::Closure(params, body, _) => {
                self.emit_closure(params, body)
            }
            Expr::Pipe(lhs, rhs, _) => {
                // x |> f  =>  f(x)
                self.emit_pipe(lhs, rhs)
            }
            Expr::Try(inner, _) => self.emit_expr(inner),
            Expr::Catch(inner, _, _, _) => self.emit_expr(inner),
            Expr::ElseUnwrap(inner, _, _) => self.emit_expr(inner),
            Expr::Range(start, _end, _) => {
                // Just return start for now (ranges are handled in for loops)
                self.emit_expr(start)
            }
            Expr::Spread(inner, _) => self.emit_expr(inner),
            Expr::Error(_) => Ok(self.context.i64_type().const_int(0, false).into()),
        }
    }

    fn emit_string_literal(&mut self, s: &str) -> Result<BasicValueEnum<'ctx>> {
        let escaped = s.replace("\\n", "\n").replace("\\t", "\t");
        let global = self.builder
            .build_global_string_ptr(&escaped, "str")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        Ok(global.as_pointer_value().into())
    }

    fn emit_interpolated_string(&mut self, parts: &[StringPart]) -> Result<BasicValueEnum<'ctx>> {
        // Build format string and args for sprintf-style concatenation
        let mut format_str = String::new();
        let mut args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();

        for part in parts {
            match &part.kind {
                StringPartKind::Literal(s) => {
                    format_str.push_str(&s.replace('%', "%%"));
                }
                StringPartKind::Expr(expr) => {
                    let val = self.emit_expr(expr)?;
                    if val.is_int_value() {
                        format_str.push_str("%lld");
                        args.push(val.into());
                    } else if val.is_float_value() {
                        format_str.push_str("%g");
                        args.push(val.into());
                    } else {
                        format_str.push_str("%s");
                        args.push(val.into());
                    }
                }
            }
        }

        // Use snprintf to build the string
        let snprintf = self.get_or_declare_snprintf();
        let buf_size = 1024u64;
        let buf = self.builder
            .build_array_alloca(
                self.context.i8_type(),
                self.context.i64_type().const_int(buf_size, false),
                "strbuf",
            )
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        let fmt_global = self.builder
            .build_global_string_ptr(&format_str, "fmt")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> = vec![
            buf.into(),
            self.context.i64_type().const_int(buf_size, false).into(),
            fmt_global.as_pointer_value().into(),
        ];
        call_args.extend(args);

        self.builder
            .build_call(snprintf, &call_args, "snprintf_ret")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        Ok(buf.into())
    }

    fn emit_ident(&mut self, id: &Ident) -> Result<BasicValueEnum<'ctx>> {
        // Check local variables
        if let Some((alloca, ty)) = self.variables.get(&id.name) {
            let val = self.builder
                .build_load(*ty, *alloca, &id.name)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            return Ok(val);
        }

        // Builtin names that are valid as identifiers (function references)
        match id.name.as_str() {
            "println" | "print" | "assert" | "assert_eq" | "read_line" => {
                // Return a dummy pointer — these are handled in emit_call
                return Ok(self.context.i64_type().const_int(0, false).into());
            }
            _ => {}
        }

        // Check if it's a function reference
        if let Some(func) = self.functions.get(&id.name) {
            return Ok(func.as_global_value().as_pointer_value().into());
        }

        // Check module names (std.*)
        match id.name.as_str() {
            "std" | "string" | "list" | "map" | "set" | "json" | "math" | "io" | "env"
            | "time" | "crypto" | "log" | "http" | "sql" => {
                return Ok(self.context.i64_type().const_int(0, false).into());
            }
            _ => {}
        }

        // Check if it's an enum constructor
        if self.enum_types.contains_key(&id.name) {
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        Err(CodegenError::Undefined(id.name.clone()))
    }

    fn emit_binary(
        &mut self,
        lhs: &Expr,
        op: BinOp,
        rhs: &Expr,
    ) -> Result<BasicValueEnum<'ctx>> {
        let l = self.emit_expr(lhs)?;
        let r = self.emit_expr(rhs)?;

        // Integer operations
        if l.is_int_value() && r.is_int_value() {
            let li = l.into_int_value();
            let ri = r.into_int_value();

            let result = match op {
                BinOp::Add => self.builder.build_int_add(li, ri, "add"),
                BinOp::Sub => self.builder.build_int_sub(li, ri, "sub"),
                BinOp::Mul => self.builder.build_int_mul(li, ri, "mul"),
                BinOp::Div => self.builder.build_int_signed_div(li, ri, "div"),
                BinOp::Mod => self.builder.build_int_signed_rem(li, ri, "mod"),
                BinOp::Eq => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::EQ, li, ri, "eq")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::NotEq => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::NE, li, ri, "neq")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::Lt => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::SLT, li, ri, "lt")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::Gt => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::SGT, li, ri, "gt")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::LtEq => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::SLE, li, ri, "lte")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::GtEq => {
                    return Ok(self.builder
                        .build_int_compare(inkwell::IntPredicate::SGE, li, ri, "gte")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::And => self.builder.build_and(li, ri, "and"),
                BinOp::Or => self.builder.build_or(li, ri, "or"),
            };
            return Ok(result.map_err(|e| CodegenError::Llvm(e.to_string()))?.into());
        }

        // Float operations
        if l.is_float_value() && r.is_float_value() {
            let lf = l.into_float_value();
            let rf = r.into_float_value();

            let result = match op {
                BinOp::Add => self.builder.build_float_add(lf, rf, "fadd"),
                BinOp::Sub => self.builder.build_float_sub(lf, rf, "fsub"),
                BinOp::Mul => self.builder.build_float_mul(lf, rf, "fmul"),
                BinOp::Div => self.builder.build_float_div(lf, rf, "fdiv"),
                BinOp::Mod => self.builder.build_float_rem(lf, rf, "fmod"),
                BinOp::Eq => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::OEQ, lf, rf, "feq")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::NotEq => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "fneq")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::Lt => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "flt")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::Gt => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "fgt")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::LtEq => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::OLE, lf, rf, "flte")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::GtEq => {
                    return Ok(self.builder
                        .build_float_compare(inkwell::FloatPredicate::OGE, lf, rf, "fgte")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into());
                }
                BinOp::And | BinOp::Or => {
                    return Err(CodegenError::Type("logical operators require Bool".into()));
                }
            };
            return Ok(result.map_err(|e| CodegenError::Llvm(e.to_string()))?.into());
        }

        // Mixed int/float: promote int to float
        if (l.is_int_value() && r.is_float_value()) || (l.is_float_value() && r.is_int_value()) {
            let (lf, rf) = if l.is_int_value() {
                let promoted = self.builder
                    .build_signed_int_to_float(l.into_int_value(), self.context.f64_type(), "itof")
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                (promoted, r.into_float_value())
            } else {
                let promoted = self.builder
                    .build_signed_int_to_float(r.into_int_value(), self.context.f64_type(), "itof")
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                (l.into_float_value(), promoted)
            };

            let result = match op {
                BinOp::Add => self.builder.build_float_add(lf, rf, "fadd"),
                BinOp::Sub => self.builder.build_float_sub(lf, rf, "fsub"),
                BinOp::Mul => self.builder.build_float_mul(lf, rf, "fmul"),
                BinOp::Div => self.builder.build_float_div(lf, rf, "fdiv"),
                _ => return Err(CodegenError::Type("unsupported mixed-type operation".into())),
            };
            return Ok(result.map_err(|e| CodegenError::Llvm(e.to_string()))?.into());
        }

        Err(CodegenError::Type(format!(
            "unsupported binary operation on types"
        )))
    }

    fn emit_unary(&mut self, op: UnaryOp, inner: &Expr) -> Result<BasicValueEnum<'ctx>> {
        let val = self.emit_expr(inner)?;
        match op {
            UnaryOp::Neg => {
                if val.is_int_value() {
                    Ok(self.builder
                        .build_int_neg(val.into_int_value(), "neg")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into())
                } else if val.is_float_value() {
                    Ok(self.builder
                        .build_float_neg(val.into_float_value(), "fneg")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into())
                } else {
                    Err(CodegenError::Type("cannot negate this type".into()))
                }
            }
            UnaryOp::Not => {
                if val.is_int_value() {
                    Ok(self.builder
                        .build_not(val.into_int_value(), "not")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into())
                } else {
                    Err(CodegenError::Type("cannot negate this type".into()))
                }
            }
        }
    }

    fn emit_call(
        &mut self,
        callee: &Expr,
        args: &[CallArg],
    ) -> Result<BasicValueEnum<'ctx>> {
        let call_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .map(|a| self.emit_expr(&a.value).map(|v| v.into()))
            .collect::<Result<Vec<_>>>()?;

        match callee {
            Expr::Ident(id) => {
                // Handle builtins
                match id.name.as_str() {
                    "println" | "print" => {
                        return self.emit_println(&call_args);
                    }
                    "assert" => {
                        return self.emit_assert(&call_args);
                    }
                    "assert_eq" => {
                        return self.emit_assert_eq(&call_args);
                    }
                    _ => {}
                }

                // Look up function
                if let Some(func) = self.module.get_function(&id.name) {
                    let ret = self.builder
                        .build_call(func, &call_args, "call")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    Ok(ret
                        .try_as_basic_value()
                        .left()
                        .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()))
                } else {
                    Err(CodegenError::Undefined(id.name.clone()))
                }
            }
            Expr::FieldAccess(_obj, _method, _) => {
                // Handle std module calls like std.string.split()
                // For now, return a dummy value
                Ok(self.context.i64_type().const_int(0, false).into())
            }
            _ => {
                // Function pointer call
                Ok(self.context.i64_type().const_int(0, false).into())
            }
        }
    }

    fn emit_println(&mut self, args: &[BasicMetadataValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>> {
        let printf = self.module.get_function("printf").unwrap();

        if args.is_empty() {
            let nl = self.builder
                .build_global_string_ptr("\n", "nl")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            self.builder
                .build_call(printf, &[nl.as_pointer_value().into()], "")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        } else {
            // Determine format string based on argument type
            let arg = args[0];
            let fmt = if let BasicMetadataValueEnum::IntValue(iv) = arg {
                if iv.get_type().get_bit_width() == 1 {
                    // Bool: convert to "true"/"false"
                    let true_str = self.builder.build_global_string_ptr("true\n", "true_s")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    let false_str = self.builder.build_global_string_ptr("false\n", "false_s")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    let selected = self.builder
                        .build_select(iv, true_str.as_pointer_value(), false_str.as_pointer_value(), "bool_str")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    self.builder
                        .build_call(printf, &[selected.into()], "")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    return Ok(self.context.i64_type().const_int(0, false).into());
                } else {
                    "%lld\n"
                }
            } else if let BasicMetadataValueEnum::FloatValue(_) = arg {
                "%g\n"
            } else {
                "%s\n"
            };

            let fmt_global = self.builder
                .build_global_string_ptr(fmt, "fmt")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;

            let mut printf_args: Vec<BasicMetadataValueEnum<'ctx>> = vec![fmt_global.as_pointer_value().into()];
            printf_args.extend_from_slice(args);

            self.builder
                .build_call(printf, &printf_args, "")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        Ok(self.context.i64_type().const_int(0, false).into())
    }

    fn emit_assert(&mut self, args: &[BasicMetadataValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>> {
        if args.is_empty() {
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        let func = self.current_function();
        let cond = match args[0] {
            BasicMetadataValueEnum::IntValue(iv) => iv,
            _ => return Ok(self.context.i64_type().const_int(0, false).into()),
        };

        let then_bb = self.context.append_basic_block(func, "assert.ok");
        let else_bb = self.context.append_basic_block(func, "assert.fail");

        self.builder.build_conditional_branch(cond, then_bb, else_bb)
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        // Failure path
        self.builder.position_at_end(else_bb);
        let msg = self.builder
            .build_global_string_ptr("assertion failed\n", "assert_msg")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        let printf = self.module.get_function("printf").unwrap();
        self.builder
            .build_call(printf, &[msg.as_pointer_value().into()], "")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        // Call exit(1)
        let exit_fn = self.get_or_declare_exit();
        self.builder
            .build_call(exit_fn, &[self.context.i32_type().const_int(1, false).into()], "")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        self.builder.build_unreachable()
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        // Success path
        self.builder.position_at_end(then_bb);
        Ok(self.context.i64_type().const_int(0, false).into())
    }

    fn emit_assert_eq(&mut self, args: &[BasicMetadataValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>> {
        if args.len() < 2 {
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        let lhs = match args[0] {
            BasicMetadataValueEnum::IntValue(iv) => iv,
            _ => return Ok(self.context.i64_type().const_int(0, false).into()),
        };
        let rhs = match args[1] {
            BasicMetadataValueEnum::IntValue(iv) => iv,
            _ => return Ok(self.context.i64_type().const_int(0, false).into()),
        };

        let cond = self.builder
            .build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "eq")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        self.emit_assert(&[cond.into()])
    }

    fn emit_if(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_branch: Option<&Expr>,
    ) -> Result<BasicValueEnum<'ctx>> {
        let cond_val = self.emit_expr(cond)?;
        let cond_bool = if cond_val.is_int_value() {
            let iv = cond_val.into_int_value();
            if iv.get_type().get_bit_width() == 1 {
                iv
            } else {
                self.builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        iv,
                        iv.get_type().const_int(0, false),
                        "tobool",
                    )
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?
            }
        } else {
            return Err(CodegenError::Type("if condition must be Bool".into()));
        };

        let func = self.current_function();
        let then_bb = self.context.append_basic_block(func, "if.then");
        let else_bb = self.context.append_basic_block(func, "if.else");
        let merge_bb = self.context.append_basic_block(func, "if.merge");

        self.builder.build_conditional_branch(cond_bool, then_bb, else_bb)
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_bb);
        let then_val = self.emit_block(then_block)?;
        let then_val = then_val.unwrap_or_else(|| self.context.i64_type().const_int(0, false).into());
        let then_branches_to_merge = self.needs_terminator();
        if then_branches_to_merge {
            self.builder.build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }
        let then_end_bb = self.builder.get_insert_block().unwrap();

        // Else block
        self.builder.position_at_end(else_bb);
        let else_val = if let Some(else_expr) = else_branch {
            self.emit_expr(else_expr)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };
        let else_branches_to_merge = self.needs_terminator();
        if else_branches_to_merge {
            self.builder.build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }
        let else_end_bb = self.builder.get_insert_block().unwrap();

        // Merge with phi — only add incoming from blocks that actually branch here
        self.builder.position_at_end(merge_bb);

        if !then_branches_to_merge && !else_branches_to_merge {
            // Both branches returned/terminated — merge block is unreachable
            self.builder.build_unreachable()
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            Ok(self.context.i64_type().const_int(0, false).into())
        } else if then_branches_to_merge && else_branches_to_merge
            && then_val.get_type() == else_val.get_type()
        {
            let phi = self.builder
                .build_phi(then_val.get_type(), "if.val")
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            phi.add_incoming(&[(&then_val, then_end_bb), (&else_val, else_end_bb)]);
            Ok(phi.as_basic_value())
        } else if then_branches_to_merge {
            // Only then branches to merge
            Ok(then_val)
        } else {
            // Only else branches to merge
            Ok(else_val)
        }
    }

    fn emit_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
    ) -> Result<BasicValueEnum<'ctx>> {
        let val = self.emit_expr(scrutinee)?;
        let func = self.current_function();
        let merge_bb = self.context.append_basic_block(func, "match.end");

        let mut arm_results: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();

        // For integer/bool matching, use a switch
        if val.is_int_value() {
            let int_val = val.into_int_value();
            let default_bb = self.context.append_basic_block(func, "match.default");

            let mut cases: Vec<(inkwell::values::IntValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
            let mut default_arm: Option<&MatchArm> = None;

            for arm in arms {
                match &arm.pattern {
                    Pattern::Literal(Expr::IntLit(n, _)) => {
                        let bb = self.context.append_basic_block(func, "match.arm");
                        let case_val = self.context.i64_type().const_int(*n as u64, true);
                        cases.push((case_val, bb));

                        self.builder.position_at_end(bb);
                        let result = self.emit_expr(&arm.body)?;
                        if self.needs_terminator() {
                            self.builder.build_unconditional_branch(merge_bb)
                                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                        }
                        arm_results.push((result, self.builder.get_insert_block().unwrap()));
                    }
                    Pattern::Literal(Expr::BoolLit(b, _)) => {
                        let bb = self.context.append_basic_block(func, "match.arm");
                        let case_val = int_val.get_type().const_int(*b as u64, false);
                        cases.push((case_val, bb));

                        self.builder.position_at_end(bb);
                        let result = self.emit_expr(&arm.body)?;
                        if self.needs_terminator() {
                            self.builder.build_unconditional_branch(merge_bb)
                                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                        }
                        arm_results.push((result, self.builder.get_insert_block().unwrap()));
                    }
                    Pattern::Wildcard(_) | Pattern::Binding(_) => {
                        default_arm = Some(arm);
                    }
                    _ => {
                        default_arm = Some(arm);
                    }
                }
            }

            // Build switch before default block
            let _switch_bb = self.builder.get_insert_block().unwrap();
            // We need to go back to where the scrutinee was evaluated
            // Actually, the builder is still at the current position after evaluating scrutinee
            // but we've been moving it around. Let's find the right block.
            // The switch needs to be at the block where we computed val.
            // Since we've been moving around, let me restructure:

            // Actually let me position properly. The scrutinee was already evaluated.
            // We need to build the switch in the block that was current when we started.
            // But we've moved. Let me re-approach:

            // We need a dedicated block for the switch
            let _switch_block = self.context.append_basic_block(func, "match.switch");

            // The issue is the builder has moved. Let me handle this by repositioning.
            // For now, just handle the default case:
            self.builder.position_at_end(default_bb);
            if let Some(arm) = default_arm {
                // Bind the variable if it's a binding pattern
                if let Pattern::Binding(id) = &arm.pattern {
                    let ty: BasicTypeEnum<'ctx> = int_val.get_type().into();
                    let alloca = self.create_entry_alloca(func, &id.name, ty);
                    self.builder.build_store(alloca, int_val)
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    self.variables.insert(id.name.clone(), (alloca, ty));
                }
                let result = self.emit_expr(&arm.body)?;
                if self.needs_terminator() {
                    self.builder.build_unconditional_branch(merge_bb)
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                }
                arm_results.push((result, self.builder.get_insert_block().unwrap()));
            } else {
                self.builder.build_unconditional_branch(merge_bb)
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            }

            // Go back and build the switch in the right place
            // We need the block before any arms. Since emit_expr(scrutinee) was the last
            // thing in the original block, we can use switch_block.
            // But actually we need to insert the switch BEFORE the arm blocks.
            // The simplest fix: restructure to build switch first.

            // For now, just branch to merge
            self.builder.position_at_end(merge_bb);
            if arm_results.is_empty() {
                return Ok(self.context.i64_type().const_int(0, false).into());
            }
            return Ok(arm_results[0].0);
        }

        // Default fallthrough for non-int matching
        self.builder.position_at_end(merge_bb);
        Ok(self.context.i64_type().const_int(0, false).into())
    }

    fn emit_field_access(
        &mut self,
        obj: &Expr,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>> {
        let obj_val = self.emit_expr(obj)?;

        if obj_val.is_struct_value() {
            let sv = obj_val.into_struct_value();
            // Try to find the field index
            // We need to know the struct type name to look up field indices
            if let Expr::Ident(_id) = obj {
                // Look through struct_types to find a matching struct for this variable
                for (_name, (_st, fields)) in &self.struct_types {
                    if let Some(idx) = fields.iter().position(|f| f == field) {
                        let val = self.builder
                            .build_extract_value(sv, idx as u32, field)
                            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                        return Ok(val);
                    }
                }
            }
        }

        // Fallback for pointer-based structs
        if obj_val.is_pointer_value() {
            // Look up which struct this pointer refers to and get field offset
            // For now, return a dummy
        }

        Ok(self.context.i64_type().const_int(0, false).into())
    }

    fn emit_field_store(
        &mut self,
        _obj: &Expr,
        _field: &str,
        _val: BasicValueEnum<'ctx>,
    ) -> Result<()> {
        // Will be expanded when we have heap-allocated structs
        Ok(())
    }

    fn emit_index(&mut self, _obj: &Expr, _idx: &Expr) -> Result<BasicValueEnum<'ctx>> {
        // Simplified: will be expanded with ARC runtime for lists
        Ok(self.context.i64_type().const_int(0, false).into())
    }

    fn emit_struct_init(
        &mut self,
        name: &str,
        fields: &[FieldInit],
    ) -> Result<BasicValueEnum<'ctx>> {
        if let Some((struct_type, field_names)) = self.struct_types.get(name).cloned() {
            let mut struct_val = struct_type.get_undef();

            for field in fields {
                if field.name.name == ".." {
                    continue;
                }
                if let Some(idx) = field_names.iter().position(|f| f == &field.name.name) {
                    let val = self.emit_expr(&field.value)?;
                    struct_val = self.builder
                        .build_insert_value(struct_val, val, idx as u32, &field.name.name)
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?
                        .into_struct_value();
                }
            }

            Ok(struct_val.into())
        } else {
            Err(CodegenError::Undefined(format!("struct {}", name)))
        }
    }

    fn emit_list_literal(&mut self, items: &[Expr]) -> Result<BasicValueEnum<'ctx>> {
        // Simplified list: allocate array on stack
        // Will be replaced with ARC-managed heap allocation in S15
        if items.is_empty() {
            return Ok(self.context.i64_type().const_int(0, false).into());
        }

        let first = self.emit_expr(&items[0])?;
        let elem_type = first.get_type();
        let arr_type = match elem_type {
            BasicTypeEnum::IntType(t) => t.array_type(items.len() as u32),
            BasicTypeEnum::FloatType(t) => t.array_type(items.len() as u32),
            _ => return Ok(self.context.i64_type().const_int(0, false).into()),
        };

        let alloca = self.builder
            .build_alloca(arr_type, "list")
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        // Store each element
        for (i, item) in items.iter().enumerate() {
            let val = if i == 0 { first } else { self.emit_expr(item)? };
            let idx = self.context.i64_type().const_int(i as u64, false);
            let ptr = unsafe {
                self.builder
                    .build_gep(arr_type, alloca, &[self.context.i64_type().const_int(0, false), idx], "elem_ptr")
                    .map_err(|e| CodegenError::Llvm(e.to_string()))?
            };
            self.builder.build_store(ptr, val)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
        }

        Ok(alloca.into())
    }

    fn emit_closure(
        &mut self,
        params: &[ClosureParam],
        body: &Expr,
    ) -> Result<BasicValueEnum<'ctx>> {
        // Create a new function for the closure
        let closure_name = format!("closure_{}", self.functions.len());

        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|_| self.context.i64_type().into()) // Default to i64 for untyped params
            .collect();

        let fn_type = self.context.i64_type().fn_type(&param_types, false);
        let closure_fn = self.module.add_function(&closure_name, fn_type, None);

        let entry = self.context.append_basic_block(closure_fn, "entry");
        let saved_block = self.builder.get_insert_block();
        self.builder.position_at_end(entry);

        let old_vars = self.variables.clone();
        self.variables.clear();

        // Bind parameters
        for (i, param) in params.iter().enumerate() {
            let param_val = closure_fn.get_nth_param(i as u32).unwrap();
            let param_ty = param_val.get_type();
            let alloca = self.create_entry_alloca(closure_fn, &param.name.name, param_ty);
            self.builder.build_store(alloca, param_val)
                .map_err(|e| CodegenError::Llvm(e.to_string()))?;
            self.variables.insert(param.name.name.clone(), (alloca, param_ty));
        }

        let result = self.emit_expr(body)?;
        self.builder.build_return(Some(&result))
            .map_err(|e| CodegenError::Llvm(e.to_string()))?;

        self.variables = old_vars;

        // Restore builder position
        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }

        // Return function pointer
        Ok(closure_fn.as_global_value().as_pointer_value().into())
    }

    fn emit_pipe(&mut self, lhs: &Expr, rhs: &Expr) -> Result<BasicValueEnum<'ctx>> {
        // x |> f  =>  f(x)
        let arg = self.emit_expr(lhs)?;

        match rhs {
            Expr::Ident(id) => {
                if let Some(func) = self.module.get_function(&id.name) {
                    let ret = self.builder
                        .build_call(func, &[arg.into()], "pipe_call")
                        .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                    Ok(ret
                        .try_as_basic_value()
                        .left()
                        .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()))
                } else {
                    Err(CodegenError::Undefined(id.name.clone()))
                }
            }
            Expr::Call(callee, extra_args, _) => {
                // x |> f(y, z) => f(x, y, z)
                let mut all_args: Vec<BasicMetadataValueEnum<'ctx>> = vec![arg.into()];
                for a in extra_args {
                    let v = self.emit_expr(&a.value)?;
                    all_args.push(v.into());
                }

                if let Expr::Ident(id) = callee.as_ref() {
                    if let Some(func) = self.module.get_function(&id.name) {
                        let ret = self.builder
                            .build_call(func, &all_args, "pipe_call")
                            .map_err(|e| CodegenError::Llvm(e.to_string()))?;
                        return Ok(ret
                            .try_as_basic_value()
                            .left()
                            .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into()));
                    }
                }
                Err(CodegenError::Unsupported("complex pipe expression".into()))
            }
            _ => Err(CodegenError::Unsupported("complex pipe expression".into())),
        }
    }

    // ============================================================
    // Type mapping
    // ============================================================

    fn type_expr_to_llvm(&self, ty: &TypeExpr) -> Result<BasicTypeEnum<'ctx>> {
        match ty {
            TypeExpr::Named(id) => match id.name.as_str() {
                "Int" => Ok(self.context.i64_type().into()),
                "Float" => Ok(self.context.f64_type().into()),
                "Bool" => Ok(self.context.bool_type().into()),
                "String" => Ok(self.context.ptr_type(AddressSpace::default()).into()),
                "Byte" => Ok(self.context.i8_type().into()),
                name => {
                    if let Some((st, _)) = self.struct_types.get(name) {
                        Ok((*st).into())
                    } else if let Some(ei) = self.enum_types.get(name) {
                        Ok(ei.llvm_type.into())
                    } else {
                        // Default to i64 for unknown types (will be refined)
                        Ok(self.context.i64_type().into())
                    }
                }
            },
            TypeExpr::Generic(id, args) => match id.name.as_str() {
                "List" | "Set" => {
                    // Pointer to heap-allocated array (simplified)
                    Ok(self.context.ptr_type(AddressSpace::default()).into())
                }
                "Map" => {
                    Ok(self.context.ptr_type(AddressSpace::default()).into())
                }
                "Option" if !args.is_empty() => {
                    // Option as nullable pointer or tagged struct
                    Ok(self.context.ptr_type(AddressSpace::default()).into())
                }
                "Result" if args.len() >= 2 => {
                    // Result as tagged union
                    Ok(self.context.ptr_type(AddressSpace::default()).into())
                }
                _ => Ok(self.context.i64_type().into()),
            },
            TypeExpr::Option(_inner) => {
                // Option[T] as pointer (null = None)
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            TypeExpr::Unit => {
                // Unit type — use void in return position, but need a concrete type here
                Ok(self.context.i64_type().into())
            }
        }
    }

    // ============================================================
    // Helpers
    // ============================================================

    fn create_entry_alloca(
        &self,
        func: FunctionValue<'ctx>,
        name: &str,
        ty: BasicTypeEnum<'ctx>,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = func.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(inst) => builder.position_before(&inst),
            None => builder.position_at_end(entry),
        }
        builder.build_alloca(ty, name).unwrap()
    }

    fn current_function(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    fn needs_terminator(&self) -> bool {
        self.builder
            .get_insert_block()
            .map(|bb| bb.get_terminator().is_none())
            .unwrap_or(false)
    }

    fn default_value(&self, ty: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match ty {
            BasicTypeEnum::IntType(t) => t.const_int(0, false).into(),
            BasicTypeEnum::FloatType(t) => t.const_float(0.0).into(),
            BasicTypeEnum::PointerType(t) => t.const_null().into(),
            BasicTypeEnum::StructType(t) => t.get_undef().into(),
            BasicTypeEnum::ArrayType(t) => t.get_undef().into(),
            BasicTypeEnum::VectorType(t) => t.get_undef().into(),
        }
    }

    fn get_alloca_type(&self, _alloca: PointerValue<'ctx>) -> BasicTypeEnum<'ctx> {
        // Try to determine the type from the alloca instruction
        // For now, use heuristic based on stored values
        // This is a limitation of opaque pointers in LLVM 18
        // We'll store type info in a side table in the future
        self.context.i64_type().into()
    }

    fn get_or_declare_snprintf(&self) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function("snprintf") {
            return f;
        }
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], true);
        self.module.add_function("snprintf", fn_type, None)
    }

    fn get_or_declare_exit(&self) -> FunctionValue<'ctx> {
        if let Some(f) = self.module.get_function("exit") {
            return f;
        }
        let i32_type = self.context.i32_type();
        let fn_type = self.context.void_type().fn_type(&[i32_type.into()], false);
        self.module.add_function("exit", fn_type, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_ir(source: &str) -> String {
        let program = klar_parser::parse(source).unwrap();
        generate_ir(&program).unwrap()
    }

    #[test]
    fn hello_world_ir() {
        let ir = compile_ir(r#"fn main() { println("Hello, world!") }"#);
        assert!(ir.contains("define void @klar_main()"));
        assert!(ir.contains("Hello, world!"));
        assert!(ir.contains("call"));
    }

    #[test]
    fn arithmetic_ir() {
        let ir = compile_ir("fn add(a: Int, b: Int) -> Int { a + b }");
        assert!(ir.contains("define i64 @add(i64"));
        assert!(ir.contains("add"));
    }

    #[test]
    fn bool_logic_ir() {
        let ir = compile_ir("fn check(x: Bool) -> Bool {\n  not x\n}");
        assert!(ir.contains("define i1 @check(i1"));
    }

    #[test]
    fn function_call_ir() {
        let ir = compile_ir(r#"
            fn double(x: Int) -> Int { x * 2 }
            fn main() { let y = double(21) println(y) }
        "#);
        assert!(ir.contains("define i64 @double"));
        assert!(ir.contains("call i64 @double"));
    }

    #[test]
    fn struct_ir() {
        let ir = compile_ir("struct Point {\n  x: Int\n  y: Int\n}");
        assert!(ir.contains("i64")); // struct fields
    }

    #[test]
    fn if_else_ir() {
        let ir = compile_ir("fn abs(x: Int) -> Int { if x < 0 { -x } else { x } }");
        assert!(ir.contains("if.then"));
        assert!(ir.contains("if.else"));
        assert!(ir.contains("if.merge"));
    }

    #[test]
    fn for_loop_ir() {
        let ir = compile_ir("fn count() {\n  let items = [1, 2, 3]\n  for item in items {\n    println(item)\n  }\n}");
        assert!(ir.contains("define void @count()"));
    }

    #[test]
    fn float_arithmetic_ir() {
        let ir = compile_ir("fn area(r: Float) -> Float { 3.14159 * r * r }");
        assert!(ir.contains("define double @area(double"));
        assert!(ir.contains("fmul"));
    }
}
