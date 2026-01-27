import { defineWorkersConfig } from "@cloudflare/vitest-pool-workers/config";

export default defineWorkersConfig({
	test: {
		globals: true,
		testTimeout: 60000,
	},
	// Cloudflare Workers pool configuration (Vitest 4.x format)
	poolOptions: {
		workers: {
			wrangler: {
				configPath: "./wrangler.toml",
			},
		},
	},
});
