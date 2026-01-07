import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
    plugins: [
        svelte(),
        svelteTesting()
    ],
    test: {
        include: ['src/**/*.{test,spec}.{js,ts}'],
        globals: true,
        environment: 'happy-dom',
        alias: {
            $lib: '/src/lib',
            '$app/environment': '/src/lib/__mocks__/app-environment.ts',
            '$app/stores': '/src/lib/__mocks__/app-stores.ts',
            '$app/navigation': '/src/lib/__mocks__/app-navigation.ts',
            '@tauri-apps/plugin-http': '/src/lib/__mocks__/tauri-plugin-http.ts'
        },
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            include: ['src/lib/**/*.ts'],
            exclude: ['src/lib/**/*.test.ts', 'src/lib/**/*.spec.ts'],
            thresholds: {
                lines: 50,
                functions: 50,
                branches: 50,
                statements: 50
            }
        }
    },
    resolve: {
        alias: {
            $lib: '/src/lib'
        }
    }
});
