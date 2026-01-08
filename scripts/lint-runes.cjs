#!/usr/bin/env node
/**
 * Lint script to detect $state/$derived/$effect runes in .ts files
 * These runes are only allowed in .svelte and .svelte.ts files
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const RUNES = ['$state', '$derived', '$effect', '$props', '$bindable', '$inspect'];
const ALLOWED_EXTENSIONS = ['.svelte', '.svelte.ts', '.svelte.js'];
const SEARCH_DIRS = ['src/lib', 'src/routes'];
const IGNORE_PATTERNS = ['node_modules', '.svelte-kit', 'dist', 'build', '*.test.ts', '*.spec.ts'];

let errors = [];
let warnings = [];

function isAllowedFile(filePath) {
  return ALLOWED_EXTENSIONS.some(ext => filePath.endsWith(ext));
}

function shouldIgnore(filePath) {
  return IGNORE_PATTERNS.some(pattern => {
    if (pattern.startsWith('*')) {
      return filePath.endsWith(pattern.slice(1));
    }
    return filePath.includes(pattern);
  });
}

function checkFile(filePath) {
  if (shouldIgnore(filePath)) return;
  if (isAllowedFile(filePath)) return;
  if (!filePath.endsWith('.ts') && !filePath.endsWith('.js')) return;
  
  const content = fs.readFileSync(filePath, 'utf-8');
  const lines = content.split('\n');
  
  lines.forEach((line, index) => {
    RUNES.forEach(rune => {
      // Match rune usage (not in comments or strings)
      const runeRegex = new RegExp(`(?<!['"\`/])\\${rune}(?:\\s*[<(]|\\s*=)`, 'g');
      if (runeRegex.test(line)) {
        errors.push({
          file: filePath,
          line: index + 1,
          rune,
          message: `${rune} rune found in .ts file. Rename to .svelte.ts`,
          content: line.trim().slice(0, 80)
        });
      }
    });
  });
}

function walkDir(dir) {
  if (!fs.existsSync(dir)) return;
  
  const files = fs.readdirSync(dir);
  files.forEach(file => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory()) {
      if (!shouldIgnore(filePath)) {
        walkDir(filePath);
      }
    } else {
      checkFile(filePath);
    }
  });
}

// Also check for missing Tauri plugin imports
function checkTauriImports(dir) {
  if (!fs.existsSync(dir)) return;
  
  const files = fs.readdirSync(dir, { recursive: true });
  files.forEach(file => {
    const filePath = path.join(dir, file);
    if (!fs.statSync(filePath).isFile()) return;
    if (!filePath.endsWith('.ts') && !filePath.endsWith('.svelte')) return;
    if (shouldIgnore(filePath)) return;
    
    const content = fs.readFileSync(filePath, 'utf-8');
    
    // Check for @tauri-apps/plugin-* imports that might not be installed
    const pluginImports = content.match(/@tauri-apps\/plugin-[a-z-]+/g);
    if (pluginImports) {
      pluginImports.forEach(plugin => {
        // Check if plugin is in package.json
        try {
          const pkgJson = JSON.parse(fs.readFileSync('package.json', 'utf-8'));
          const deps = { ...pkgJson.dependencies, ...pkgJson.devDependencies };
          if (!deps[plugin]) {
            warnings.push({
              file: filePath,
              message: `Import "${plugin}" not found in package.json`,
              suggestion: `Run: pnpm add ${plugin}`
            });
          }
        } catch {}
      });
    }
  });
}

console.log('🔍 Checking for Svelte 5 runes in .ts files...\n');

SEARCH_DIRS.forEach(dir => walkDir(dir));
SEARCH_DIRS.forEach(dir => checkTauriImports(dir));

if (errors.length > 0) {
  console.log('❌ ERRORS: Runes found in .ts files (must be .svelte.ts)\n');
  errors.forEach(err => {
    console.log(`  ${err.file}:${err.line}`);
    console.log(`    ${err.rune} rune detected`);
    console.log(`    > ${err.content}`);
    console.log('');
  });
}

if (warnings.length > 0) {
  console.log('⚠️  WARNINGS:\n');
  warnings.forEach(warn => {
    console.log(`  ${warn.file}`);
    console.log(`    ${warn.message}`);
    if (warn.suggestion) {
      console.log(`    💡 ${warn.suggestion}`);
    }
    console.log('');
  });
}

if (errors.length === 0 && warnings.length === 0) {
  console.log('✅ All checks passed!\n');
  process.exit(0);
} else {
  console.log(`\nFound ${errors.length} error(s) and ${warnings.length} warning(s)`);
  process.exit(errors.length > 0 ? 1 : 0);
}
