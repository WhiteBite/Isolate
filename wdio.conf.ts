import type { Options } from '@wdio/types';
import path from 'path';
import fs from 'fs';

/**
 * WebdriverIO configuration for Tauri E2E tests with tauri-driver
 * 
 * Prerequisites:
 * 1. Install tauri-driver: cargo install tauri-driver
 * 2. Build the Tauri app: pnpm tauri build
 * 3. Run tests: pnpm test:e2e:tauri
 * 
 * @see https://tauri.app/v1/guides/testing/webdriver/
 */

// Path to the built Tauri application
const tauriAppPath = path.resolve(
    __dirname,
    'src-tauri/target/release/Isolate.exe'
);

// Check if the app exists
const appExists = fs.existsSync(tauriAppPath);

export const config: Options.Testrunner = {
    //
    // ====================
    // Runner Configuration
    // ====================
    runner: 'local',
    autoCompileOpts: {
        autoCompile: true,
        tsNodeOpts: {
            project: './tsconfig.json',
            transpileOnly: true,
        },
    },

    //
    // ==================
    // Specify Test Files
    // ==================
    specs: ['./tests/e2e-tauri/**/*.spec.ts'],
    exclude: [],

    //
    // ============
    // Capabilities
    // ============
    maxInstances: 1, // Tauri tests must run sequentially
    capabilities: [
        {
            // Use tauri-driver as the WebDriver server
            'tauri:options': {
                application: tauriAppPath,
            },
        },
    ],

    //
    // ===================
    // Test Configurations
    // ===================
    logLevel: 'info',
    bail: 0,
    waitforTimeout: 10000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,

    //
    // =====================
    // Services Configuration
    // =====================
    services: [
        [
            'tauri',
            {
                // tauri-driver will be started automatically
            },
        ],
    ],

    //
    // ====================
    // Framework Configuration
    // ====================
    framework: 'mocha',
    reporters: [
        'spec',
        [
            'html',
            {
                outputDir: './wdio-report',
                filename: 'report.html',
                reportTitle: 'Isolate E2E Test Report',
            },
        ],
    ],
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000,
    },

    //
    // =====
    // Hooks
    // =====
    onPrepare: function () {
        if (!appExists) {
            console.error('\n');
            console.error('='.repeat(60));
            console.error('ERROR: Tauri application not found!');
            console.error(`Expected path: ${tauriAppPath}`);
            console.error('');
            console.error('Please build the application first:');
            console.error('  pnpm tauri build');
            console.error('='.repeat(60));
            console.error('\n');
            process.exit(1);
        }
        console.log('\n🚀 Starting Tauri E2E tests with tauri-driver...\n');
    },

    beforeSession: function () {
        console.log('📱 Launching Tauri application...');
    },

    afterSession: function () {
        console.log('✅ Test session completed');
    },

    onComplete: function () {
        console.log('\n📊 All tests completed. Report available in ./wdio-report/\n');
    },
};
