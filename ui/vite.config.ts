/// <reference types="vitest/config" />

import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			'/api': 'http://localhost:3000',
			'/ws': {
				target: 'ws://localhost:3000',
				ws: true
			}
		}
	},
	test: {
		environment: 'node',
		include: ['src/**/*.test.ts'],
	}
});
