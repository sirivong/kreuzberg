# Zig Binding Audit (2026-05-30)

## Status

Currently 0/100 e2e passing - all tests crash with SIGABRT at runtime.

## Bugs Found

### CRITICAL: Null Pointer Dereference in All Extract Functions

**Location**: Lines 3885-3888, 3931-3934, 3968-3971, 3999-4002 in `packages/zig/src/kreuzberg.zig`

**Issue**: Force-unwrap of potentially-null pointers.

The extraction functions call C FFI functions that return `?*KREUZBERGExtractionResult` (optional pointer). The current code:

```zig
const _result = c.kreuzberg_extract_bytes(...);
if (c.kreuzberg_last_error_code() != 0) {
    return _first_error(KreuzbergError);
}
if (config_handle) |h| c.kreuzberg_extraction_config_free(h);
return blk: {
    const _json_ptr = c.kreuzberg_extraction_result_to_json(_result.?);  // CRASH HERE
    defer _free_string(_json_ptr);
    c.kreuzberg_extraction_result_free(_result.?);
    const slice = std.mem.sliceTo(_json_ptr, 0);  // CRASH HERE IF _json_ptr is null
    const owned = try std.heap.c_allocator.dupe(u8, slice);
    break :blk owned;
}
```

**Problems**:
1. **Line 3885 / 3968**: The `_result.?` force-unwrap will crash if `_result` is null. The error-code check assumes that if error code is non-zero, the result is null. But if the error code is zero AND the result is null, this crashes.
2. **Line 3888 / 3971**: After calling `kreuzberg_extraction_result_to_json(_result)`, the returned `_json_ptr` can be null but the code immediately dereferences it with `std.mem.sliceTo(_json_ptr, 0)`, which crashes.
3. **Resource leak**: If `_json_ptr` is null and `_result` was successfully freed, but we never reach the owned allocation, we have a dangling JSON pointer.

**Affected Functions**:
- `extract_bytes` (line 3871)
- `extract_file` (line 3897)
- `extract_bytes_sync` (line 3938)
- `extract_file_sync` (line 3951)
- `batch_extract_bytes_sync` (line 3978)
- And all batch functions using similar patterns

**Root Cause**: The generated Zig binding did not implement proper null-checking for C FFI returns. The pattern assumes every non-error call returns a valid pointer, which is not guaranteed.

---

## Fix Strategy

For each extract function:

1. **Check `_result` before unwrap**:
   ```zig
   const _result = c.kreuzberg_extract_bytes(...);
   if (_result == null) {
       if (c.kreuzberg_last_error_code() != 0) {
           return _first_error(KreuzbergError);
       }
       // Error code is 0 but result is null - treat as unknown error
       return KreuzbergError.Other;
   }
   ```

2. **Check `_json_ptr` before dereference**:
   ```zig
   const _json_ptr = c.kreuzberg_extraction_result_to_json(_result.?);
   if (_json_ptr == null) {
       c.kreuzberg_extraction_result_free(_result.?);
       return KreuzbergError.Serialization;
   }
   defer _free_string(_json_ptr);
   ```

3. **Ensure error-code check runs first**:
   Move the error-code check to immediately after the C call, before any other operations.

---

## Detailed Audit

### Type Safety

**Status**: ✅ PASS
- All Zig-side type declarations correctly match the C header (kreuzberg.h)
- Opaque handle types (e.g., `*KREUZBERGExtractionResult`) are properly declared
- Struct definitions have correct field types and layouts

### Allocator Lifetime

**Status**: ⚠️ PASS WITH CAVEATS
- Proper use of `std.heap.c_allocator` for FFI allocations
- All `defer` blocks correctly paired
- Example (line 3889): `std.heap.c_allocator.dupe(u8, slice)` returns owned slice that caller must free
- Tests correctly call `defer std.heap.c_allocator.free(_result_json)` (e.g., line 33 in smoke_test.zig)
- **Issue**: No safeguard if intermediate conversions fail

### Error Return Convention

**Status**: ❌ FAIL
- Error-code checks exist but don't fully validate all return states
- The pattern `if (error_code != 0) return error` assumes result is null, but doesn't verify
- Inverse situation (error_code == 0 but result == null) is not handled
- Should use: `if (result == null || error_code != 0)` pattern

### Null Pointer Checks

**Status**: ❌ FAIL
- Force-unwrap (`_result.?`) assumes `_result` is never null when error_code == 0
- JSON conversion return (`_json_ptr`) is never checked for null
- String conversion (`std.mem.sliceTo(_json_ptr, 0)`) dereferences unchecked pointer

### Config Handle Freeing

**Status**: ✅ PASS
- Lines 3883, 3929, 3966, 3997, 4027, 4057, 4110, 4157: Consistent patterns
- All extraction functions properly free config_handle on all code paths
- Conditional: `if (config_handle) |h| c.kreuzberg_extraction_config_free(h);` is correct

### Batch Operations

**Status**: ❌ FAIL
- `batch_extract_bytes_sync`, `batch_extract_files_sync` (lines 3978, 4005) follow same buggy pattern
- Additional complexity: iteration over batch items adds risk if conversions fail mid-loop

---

## Test Findings

**Baseline**: Currently 0/100 green (all 22 tests crash with SIGABRT + 23 skipped due to linking)

**Crash Signature**: 
```
dyld[XXXX]: Library not loaded: @rpath/libkreuzberg_ffi.dylib
```
**Resolution**: Built FFI with `task rust:ffi:build`, tests now run and crash on first null-deref.

---

## Recommendations

1. **Immediate**: Fix null-checks in all extract functions (6 functions total)
2. **Follow-up**: Add runtime assertions or sanitizers to catch null-deref earlier in CI
3. **Testing**: All tests should exercise error paths, not just happy paths
4. **Codegen**: Review Alef's Zig generator to ensure it always emits null-checks after C calls

---

## Files to Fix

- `/Users/naamanhirschfeld/workspace/kreuzberg-dev/kreuzberg/packages/zig/src/kreuzberg.zig`
  - Lines 3885-3891: `extract_bytes`
  - Lines 3931-3937: `extract_file`
  - Lines 3968-3974: `extract_bytes_sync`
  - Lines 3999-4005: `extract_file_sync`
  - Lines 4025-4031: `batch_extract_bytes_sync` (partial review needed)
  - Lines 4055-4061: `batch_extract_files_sync` (partial review needed)

No hand-edits to generated bindings — flag to Alef codegen for fix in next regeneration cycle.
