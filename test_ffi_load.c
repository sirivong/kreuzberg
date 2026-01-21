#include <dlfcn.h>
#include <stdio.h>

int main() {
    printf("Loading library...\n");
    void* handle = dlopen("target/release/libkreuzberg_ffi.dylib", RTLD_NOW);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        return 1;
    }
    printf("Library loaded successfully\n");

    const char* (*get_version)() = dlsym(handle, "kreuzberg_version");
    if (!get_version) {
        printf("Failed to find kreuzberg_version: %s\n", dlerror());
        return 1;
    }

    printf("Calling kreuzberg_version...\n");
    const char* version = get_version();
    printf("Version: %s\n", version);

    dlclose(handle);
    printf("Library unloaded successfully\n");
    return 0;
}
