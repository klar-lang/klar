use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Declare external C runtime functions that the generated code will call.
pub fn declare_runtime<'ctx>(context: &'ctx Context, module: &Module<'ctx>) {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let void_type = context.void_type();

    // int printf(const char *format, ...)
    let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
    module.add_function("printf", printf_type, None);

    // int snprintf(char *str, size_t size, const char *format, ...)
    let snprintf_type = i32_type.fn_type(
        &[ptr_type.into(), i64_type.into(), ptr_type.into()],
        true,
    );
    module.add_function("snprintf", snprintf_type, None);

    // int puts(const char *s)
    let puts_type = i32_type.fn_type(&[ptr_type.into()], false);
    module.add_function("puts", puts_type, None);

    // void exit(int status)
    let exit_type = void_type.fn_type(&[i32_type.into()], false);
    module.add_function("exit", exit_type, None);

    // void *malloc(size_t size)
    let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("malloc", malloc_type, None);

    // void free(void *ptr)
    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("free", free_type, None);

    // void *memcpy(void *dest, const void *src, size_t n)
    let memcpy_type = ptr_type.fn_type(
        &[ptr_type.into(), ptr_type.into(), i64_type.into()],
        false,
    );
    module.add_function("memcpy", memcpy_type, None);

    // size_t strlen(const char *s)
    let strlen_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("strlen", strlen_type, None);

    // int strcmp(const char *s1, const char *s2)
    let strcmp_type = i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("strcmp", strcmp_type, None);

    // === Klar Runtime functions ===

    // void* klar_arc_alloc(u64 size, u64 type_tag)
    let arc_alloc_type = ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("klar_arc_alloc", arc_alloc_type, None);

    // void klar_arc_retain(void* ptr)
    let arc_retain_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("klar_arc_retain", arc_retain_type, None);

    // void klar_arc_release(void* ptr)
    let arc_release_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("klar_arc_release", arc_release_type, None);

    // void* klar_list_new(u64 elem_size)
    let list_new_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("klar_list_new", list_new_type, None);

    // void klar_list_push(void* list, void* elem)
    let list_push_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("klar_list_push", list_push_type, None);

    // void* klar_list_get(void* list, u64 index)
    let list_get_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("klar_list_get", list_get_type, None);

    // u64 klar_list_len(void* list)
    let list_len_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("klar_list_len", list_len_type, None);

    // void* klar_string_from_cstr(char* cstr)
    let string_from_cstr_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("klar_string_from_cstr", string_from_cstr_type, None);

    // void* klar_int_to_str(i64 n)
    let int_to_str_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("klar_int_to_str", int_to_str_type, None);

    // void* klar_float_to_str(f64 n)
    let float_to_str_type = ptr_type.fn_type(&[context.f64_type().into()], false);
    module.add_function("klar_float_to_str", float_to_str_type, None);
}
