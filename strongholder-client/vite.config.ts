import { defineConfig } from 'vite'; // Use standard vite here if vitest/config fails
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit()
	],
	// --- Tauri Settings ---
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || '127.0.0.1',
		hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
		watch: { ignored: ["**/src-tauri/**"] },
	},
	build: {
		target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
		minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
		sourcemap: !!process.env.TAURI_DEBUG,
	},
});