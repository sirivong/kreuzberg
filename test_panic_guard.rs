// Test if panic guards work correctly
#[macro_export]
macro_rules! ffi_panic_guard {
    ($function_name:expr, $body:expr) => {{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(result) => result,
            Err(_panic_info) => {
                eprintln!("Panic caught in {}", $function_name);
                std::ptr::null_mut()
            }
        }
    }};
}

#[no_mangle]
pub extern "C" fn test_function() -> *mut i32 {
    ffi_panic_guard!("test_function", {
        panic!("Test panic");
        #[allow(unreachable_code)]
        Box::into_raw(Box::new(42))
    })
}

fn main() {
    println!("Calling test_function...");
    let result = test_function();
    if result.is_null() {
        println!("Returned null as expected");
    } else {
        println!("ERROR: Should have returned null!");
    }
}
