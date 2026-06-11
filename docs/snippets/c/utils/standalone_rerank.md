```c title="C"
#include <stdio.h>
#include <stdlib.h>
#include "kreuzberg.h"

int main(void) {
    const char *query = "How to train a dog";
    const char *documents[] = {
        "Dog training requires patience and consistency.",
        "Cats are independent animals that prefer to play alone.",
        "Bird care includes proper cage setup and regular cleaning."
    };

    const char *config_json =
        "{\"model\":{\"type\":\"preset\",\"name\":\"fast\"},\"top_k\":2}";

    char *results = NULL;
    kreuzberg_Error err = kreuzberg_rerank(query, documents, 3, config_json, &results);
    if (err != KREUZBERG_ERROR_NONE) {
        fprintf(stderr, "rerank failed: %d\n", err);
        return 1;
    }
    puts(results);
    kreuzberg_string_free(results);
    return 0;
}
```
