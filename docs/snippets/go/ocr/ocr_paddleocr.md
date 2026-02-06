```go title="Go"
package main

import (
	"log"

	"github.com/kreuzberg-dev/kreuzberg/packages/go/v4"
)

func main() {
	lang := "en"
	cfg := &kreuzberg.ExtractionConfig{
		OCR: &kreuzberg.OCRConfig{
			Backend:  "paddle-ocr",
			Language: &lang,
		},
	}

	result, err := kreuzberg.ExtractFileSync("scanned.pdf", cfg)
	if err != nil {
		log.Fatalf("extract failed: %v", err)
	}
	log.Println(len(result.Content))
}
```
