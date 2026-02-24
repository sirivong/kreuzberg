#!/usr/bin/env node
/**
 * Post-build script to fix WASI and env imports in wasm-bindgen generated JS.
 *
 * Problem: When Tesseract/Leptonica are compiled with WASI SDK and linked into
 * the wasm-bindgen output, the generated JS has:
 * 1. `import * as importN from "env"` / `import * as importN from "wasi_snapshot_preview1"`
 *    statements that can't be resolved (no such ES modules exist)
 * 2. Duplicate object keys in __wbg_get_imports() return value (JS last-key-wins
 *    means only the last import per namespace survives)
 *
 * Solution: Replace external module imports with inline stub implementations and
 * merge duplicate keys using Object.assign().
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.join(__dirname, "..", "pkg");
const jsFile = path.join(pkgDir, "kreuzberg_wasm.js");

if (!fs.existsSync(jsFile)) {
	console.log("No pkg/kreuzberg_wasm.js found, skipping WASI import fix.");
	process.exit(0);
}

let content = fs.readFileSync(jsFile, "utf-8");
const originalContent = content;

// Check if already patched (idempotent)
if (content.includes("__wasi_stubs__")) {
	console.log("WASI imports already patched, skipping.");
	process.exit(0);
}

// Check if there are any env/wasi imports to fix
if (!content.includes('from "env"') && !content.includes('from "wasi_snapshot_preview1"')) {
	console.log("No env/wasi_snapshot_preview1 imports found, skipping WASI import fix.");
	process.exit(0);
}

console.log("Fixing WASI and env imports in kreuzberg_wasm.js...\n");

// Step 1: Collect all importN identifiers and their source modules
const importPattern = /^import \* as (import\d+) from "(env|wasi_snapshot_preview1)";?$/gm;
const envImports = [];
const wasiImports = [];

for (const match of content.matchAll(importPattern)) {
	const [, varName, moduleName] = match;
	if (moduleName === "env") {
		envImports.push(varName);
	} else {
		wasiImports.push(varName);
	}
}

console.log(`Found ${envImports.length} env imports: ${envImports.join(", ")}`);
console.log(`Found ${wasiImports.length} wasi_snapshot_preview1 imports: ${wasiImports.join(", ")}`);

// Step 2: Remove all import statements for env and wasi_snapshot_preview1
content = content.replace(/^import \* as import\d+ from "(env|wasi_snapshot_preview1)";?\n/gm, "");

// Step 3: Insert stub definitions at the same location (before __wbg_get_imports)
const stubCode = `// __wasi_stubs__ - WASI and env import stubs for in-memory OCR processing
// env stubs: system() and mkstemp() are never called at runtime in WASM OCR
const __env_stubs__ = {
    system: () => -1,
    mkstemp: () => -1,
};

// WASI stubs: minimal implementations for WASI preview1 syscalls
const __wasi_stubs__ = {
    fd_close: () => 0,
    fd_read: (fd, iovs_ptr, iovs_len, nread_ptr) => 0,
    fd_write: (fd, iovs_ptr, iovs_len, nwritten_ptr) => {
        // For stdout/stderr (fd 1, 2), return success with 0 bytes written
        return 0;
    },
    fd_seek: (fd, offset_lo, offset_hi, whence, newoffset_ptr) => 0,
    fd_fdstat_get: (fd, fdstat_ptr) => 0,
    fd_fdstat_set_flags: (fd, flags) => 0,
    fd_prestat_get: (fd, prestat_ptr) => 8, // EBADF - no preopened dirs
    fd_prestat_dir_name: (fd, path_ptr, path_len) => 8, // EBADF
    environ_get: (environ_ptr, environ_buf_ptr) => 0,
    environ_sizes_get: (count_ptr, buf_size_ptr) => 0,
    clock_time_get: (clock_id, precision, time_ptr) => 0,
    path_create_directory: (fd, path_ptr, path_len) => 63, // ENOSYS
    path_filestat_get: (fd, flags, path_ptr, path_len, filestat_ptr) => 63,
    path_open: (dirfd, dirflags, path_ptr, path_len, oflags, fs_rights_base_lo, fs_rights_base_hi, fs_rights_inheriting_lo, fs_rights_inheriting_hi, fdflags, fd_ptr) => 63,
    path_remove_directory: (fd, path_ptr, path_len) => 63,
    path_unlink_file: (fd, path_ptr, path_len) => 63,
    proc_exit: (code) => { throw new Error(\`WASM proc_exit called with code \${code}\`); },
    sched_yield: () => 0,
};

`;

// Insert stubs before __wbg_get_imports function
const getImportsIdx = content.indexOf("function __wbg_get_imports()");
if (getImportsIdx === -1) {
	console.error("ERROR: Could not find __wbg_get_imports() function in kreuzberg_wasm.js");
	process.exit(1);
}
content = content.slice(0, getImportsIdx) + stubCode + content.slice(getImportsIdx);

// Step 4: Replace all importN references for env/wasi with the stub objects
for (const varName of envImports) {
	content = content.replaceAll(varName, "__env_stubs__");
}
for (const varName of wasiImports) {
	content = content.replaceAll(varName, "__wasi_stubs__");
}

// Step 5: Merge duplicate keys in the __wbg_get_imports return object
// The return block looks like:
//   return {
//       __proto__: null,
//       "./kreuzberg_wasm_bg.js": import0,
//       "env": __env_stubs__,
//       "env": __env_stubs__,
//       "wasi_snapshot_preview1": __wasi_stubs__,
//       "wasi_snapshot_preview1": __wasi_stubs__,
//       ...
//   };
// Since all env stubs point to the same object and all wasi stubs point to the same object,
// we just need to deduplicate the keys. Remove all duplicate "env" and "wasi_snapshot_preview1" lines.
const returnBlockStart = content.indexOf('"./kreuzberg_wasm_bg.js": import0,');
if (returnBlockStart !== -1) {
	const returnBlockEnd = content.indexOf("};", returnBlockStart);
	if (returnBlockEnd !== -1) {
		const returnBlock = content.slice(returnBlockStart, returnBlockEnd);

		// Remove duplicate "env" lines (keep first)
		let seenEnv = false;
		let seenWasi = false;
		const lines = returnBlock.split("\n");
		const dedupedLines = lines.filter((line) => {
			const trimmed = line.trim();
			if (trimmed.startsWith('"env"')) {
				if (seenEnv) return false;
				seenEnv = true;
				return true;
			}
			if (trimmed.startsWith('"wasi_snapshot_preview1"')) {
				if (seenWasi) return false;
				seenWasi = true;
				return true;
			}
			return true;
		});

		content = content.slice(0, returnBlockStart) + dedupedLines.join("\n") + content.slice(returnBlockEnd);
	}
}

if (content === originalContent) {
	console.log("No changes needed.");
} else {
	fs.writeFileSync(jsFile, content);
	const removedImports = envImports.length + wasiImports.length;
	console.log(`Replaced ${removedImports} external imports with inline stubs.`);
	console.log("Deduplicated import keys in __wbg_get_imports().");
	console.log("Done.");
}
