// @ts-ignore - node types missing in DevEnvironment temporarily
import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
  plugins: [svelte(), svelteTesting()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url))
    }
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: [
        '**/Sample Projects/**',
        '**/fixtures/**/documents/**',
        '**/fixtures/**/runs/**',
        '**/fixtures/**/exports/**',
        '**/documents/**',
        '**/runs/**',
        '**/exports/**',
        '**/help/**',
        '**/variables/**',
        '**/pipelines/**',
        '**/models/**',
        '**/prompts/**',
        '**/project.json'
      ]
    }
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./vitest.setup.ts']
  }
});
