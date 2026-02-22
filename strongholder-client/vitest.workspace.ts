import { playwright } from '@vitest/browser-playwright';
import { resolve } from 'path'; // 1. Import resolve

export default [
    {
        extends: 'vite.config.ts',
        // 2. FORCE the alias for the test runner
        resolve: {
            alias: {
                $lib: resolve(__dirname, './src/lib')
            }
        },
        test: {
            name: 'client',
            include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
            exclude: ['src/lib/server/**'],
            browser: {
                enabled: true,
                provider: playwright(),
                instances: [
                    { browser: 'chromium' }
                ],
            },
        },
    },
    {
        extends: 'vite.config.ts',
        // 3. FORCE the alias for the server tests too
        resolve: {
            alias: {
                $lib: resolve(__dirname, './src/lib')
            }
        },
        test: {
            name: 'server',
            environment: 'node',
            include: ['src/**/*.{test,spec}.{js,ts}'],
            exclude: ['src/**/*.svelte.{test,spec}.{js,ts}'],
        },
    }
];