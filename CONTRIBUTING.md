# Contributing to Isolate

Thank you for your interest in contributing to Isolate! This guide will help you get started.

## Getting Started

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/your-username/isolate.git
   cd isolate
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Run in development mode**
   ```bash
   pnpm tauri dev
   ```

## Development Setup

### Requirements

- **Rust** 1.70+ with `rustup`
- **Node.js** 18+
- **pnpm** 8+
- **Windows 10/11** (target platform)

### Recommended Tools

- VS Code with extensions: rust-analyzer, Svelte for VS Code, Tailwind CSS IntelliSense
- Tauri CLI: `cargo install tauri-cli`

## Project Structure

```
isolate/
├── src-tauri/src/       # Rust backend
│   ├── core/            # Business logic (strategy_engine, scoring)
│   ├── commands/        # Tauri IPC commands
│   ├── services/        # Services (registry, checker)
│   └── plugins/         # Plugin system
├── src/                 # SvelteKit frontend
│   ├── routes/          # Pages
│   └── lib/             # Components, stores, utilities
├── configs/             # Strategy and service configs
│   ├── strategies/      # YAML strategy definitions
│   └── services/        # Service configurations
└── docs/                # Documentation
```

## Code Style

### Rust

- Format with `rustfmt`: `cargo fmt`
- Lint with `clippy`: `cargo clippy`
- Use `Result<T, E>` for error handling with `thiserror`
- Document public APIs with `///` comments

### TypeScript/Svelte

- Format with Prettier: `pnpm format`
- **Svelte 5 runes mode** — use `$state`, `$derived`, `$effect`
- Do NOT use legacy syntax (`$:`, `onMount` for reactivity)

```svelte
<script lang="ts">
  // ✅ Correct: Svelte 5 runes
  let items = $state<Item[]>([]);
  let count = $derived(items.length);
  
  $effect(() => {
    loadItems();
  });
</script>
```

### Tailwind CSS

- Use Tailwind utilities, avoid inline styles
- Follow existing component patterns

## Testing

### Rust Tests
```bash
cargo test                    # Run all tests
cargo test --package isolate  # Run specific package
```

### Frontend Tests
```bash
pnpm test        # Run tests
pnpm test:watch  # Watch mode
```

### Important Notes

- Tests should find real bugs, not just pass
- Use `#[ignore]` for tests requiring network/files
- **Never** run multiple winws processes in parallel (causes BSOD)

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code style guidelines

3. **Test your changes**
   ```bash
   cargo fmt && cargo clippy
   cargo test
   pnpm format && pnpm test
   ```

4. **Commit with a clear message** (see below)

5. **Push and create a PR**
   - Describe what changes you made and why
   - Reference any related issues
   - Ensure CI passes

## Commit Messages

Use clear, descriptive commit messages:

```
<type>: <short description>

[optional body with more details]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation changes
- `style`: Formatting, no code change
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

### Examples

```
feat: add Discord strategy support
fix: resolve race condition in AppState initialization
refactor: extract process runner into separate module
docs: update strategy configuration guide
```

## Questions?

- Check existing [issues](https://github.com/aspect-build/isolate/issues)
- Open a new issue for bugs or feature requests
- Discussions welcome in PR comments

---

Thank you for contributing! 🎉
