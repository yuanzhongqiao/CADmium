import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"
import adapter from "@sveltejs/adapter-static"

const prodBasePath = "/CADmium"
let base = process.env.NODE_ENV === "production" ? prodBasePath : ""
// base = prodBasePath

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: [vitePreprocess({})],
	kit: {
		// adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
		// If your environment is not supported or you settled on a specific environment, switch out the adapter.
		// See https://kit.svelte.dev/docs/adapters for more information about adapters.
		adapter: adapter({
			pages: "dist",
			assets: "dist",
			strict: false
		}),
		paths: { base }
	},
	vitePlugin: {
		inspector: true
	}
}

export default config
