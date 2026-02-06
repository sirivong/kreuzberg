```typescript title="TypeScript"
import { extractFileSync } from '@kreuzberg/node';

const config = {
	ocr: {
		backend: 'paddle-ocr',
		language: 'en',
	},
};

const result = extractFileSync('scanned.pdf', null, config);
console.log(result.content);
```
