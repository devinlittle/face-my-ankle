import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import adapter from "@sveltejs/adapter-bun";
import { enhancedImages } from '@sveltejs/enhanced-img';

export default defineConfig({
  plugins: [
    sveltekit({
      compilerOptions: {
        // Force runes mode for the project, except for libraries. Can be removed in svelte 6.
        runes: ({ filename }) =>
          filename.split(/[/\\]/).includes('node_modules') ? undefined : true
      },
      adapter: adapter(
        {
          buildOptions: {
            compile: {
              target: 'bun-linux-arm64-musl',
              outfile: 'achilles-ui',
            },
            minify: true,
            bytecode: true,
            sourcemap: 'linked'
          }
        },
      )
    }),
    enhancedImages(),
  ]
});
