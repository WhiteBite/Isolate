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

## Working with TODO.md

`TODO.md` is the single source of truth for the project backlog. When working with tasks:

### Finding Tasks

1. Open `TODO.md` and look for unchecked items `- [ ]`
2. Tasks are prioritized: `[P0]` critical → `[P1]` high → `[P2]` medium → `[P3]` nice-to-have
3. Task types: `[UX]` interface, `[FE]` frontend, `[BE]` backend, `[ARCH]` architecture, `[DOC]` documentation

### Claiming a Task

1. Create a branch: `git checkout -b feat/task-description`
2. Work on the task following code style guidelines
3. Mark task as done in `TODO.md`: `- [x] [P1][FE] Task description ✅ Реализовано`

### Completing a Task

1. Add `✅ Реализовано` (or `✅ Done`) after the task description
2. If task has sub-items, mark them too
3. Commit the TODO.md change with your feature commit
4. Reference the task in your PR description

### Adding New Tasks

1. Add tasks to the appropriate section in `TODO.md`
2. Use the format: `- [ ] [Priority][Type] Description`
3. Include file paths and acceptance criteria when possible
4. For inline code TODOs, also add to section 20 "Inline TODO/FIXME"

### Syncing with Other Files

- `TODO_v2.md` and `TODO_REDESIGN.md` contain detailed specs
- `TODO.md` is the canonical task list
- Sync periodically: move completed items, add new tasks from specs

## Questions?

- Check existing [issues](https://github.com/aspect-build/isolate/issues)
- Open a new issue for bugs or feature requests
- Discussions welcome in PR comments

---

Thank you for contributing! 🎉
