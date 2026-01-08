# Plugin Manifest Contract Synchronization

## Summary

Successfully synchronized `PluginManifest.contributes` contracts between Rust and TypeScript.

## Changes Made

### 1. TypeScript Types (`src/lib/types/plugin.ts`)

#### Added Missing Types
- **`HostlistDefinition`** - Matches Rust `HostlistDefinition`
  - Fields: `id`, `name`, `description`, `format`, `domains`, `file`, `update_url`, `update_interval`, `category`, `tags`
  - Format enum: `'plain' | 'wildcard' | 'regex'`

- **`StrategyDefinition`** - Matches Rust legacy `StrategyDefinition`
  - Fields: `id`, `name`, `family`, `config_file`

- **`PluginStrategyDefinition`** - Matches Rust full strategy definition
  - Complete strategy configuration with profiles, ports, engine settings

#### Updated `PluginContributes`
Renamed from `UIPluginContributes` to `PluginContributes` for consistency.

**Before:**
```typescript
export interface UIPluginContributes {
  widgets?: WidgetDefinition[];
  settings?: SettingsPanelDefinition[];
  services?: ServiceDefinition[];
}
```

**After:**
```typescript
export interface PluginContributes {
  services?: ServiceDefinition[];      // Rust: Vec<ServiceDefinition>
  hostlists?: HostlistDefinition[];    // Rust: Vec<HostlistDefinition>
  strategies?: StrategyDefinition[];   // Rust: Vec<StrategyDefinition>
  widgets?: WidgetDefinition[];        // TypeScript only (UI plugins)
  settings?: SettingsPanelDefinition[]; // TypeScript only (UI plugins)
}
```

#### Updated `ServiceEndpoint`
**Before:**
```typescript
export interface ServiceEndpoint {
  id?: string;
  url: string;
  method: 'GET' | 'HEAD' | 'POST';
  expectedStatus?: number;  // Not in Rust
  timeout?: number;         // Not in Rust
}
```

**After:**
```typescript
export interface ServiceEndpoint {
  id: string;      // Required (matches Rust)
  name: string;    // Required (matches Rust)
  url: string;
  method?: 'GET' | 'HEAD' | 'POST'; // Optional with default 'GET'
}
```

#### Updated `PluginManifest`
- Made `author` required (matches Rust)
- Made `contributes` optional (matches Rust `#[serde(default)]`)
- Added legacy fields: `service`, `strategy`, `hostlist` (matches Rust)

#### Updated `PluginPermissions`
Added Rust fields:
- `filesystem?: boolean` - Access to filesystem
- `process?: boolean` - Access to process spawning

### 2. Rust Documentation (`src-tauri/src/plugins/manifest.rs`)

Added TypeScript correspondence comments to all major types:

```rust
/// **TypeScript Correspondence**: `PluginContributes` in `src/lib/types/plugin.ts`
/// - Rust fields: services, hostlists, strategies
/// - TypeScript additional fields: widgets, settings (for UI plugins)
```

### 3. Fixed TypeScript Code

#### `src/lib/plugins/index.ts`
- Changed export from `UIPluginContributes` to `PluginContributes`
- Added exports for `HostlistDefinition`, `StrategyDefinition`

#### `src/lib/plugins/loader.ts`
- Added optional chaining: `manifest.contributes?.widgets`
- Added optional chaining: `manifest.contributes?.settings`

#### `src/lib/plugins/plugins.test.ts`
- Fixed `ServiceEndpoint` structure (added `id`, `name`, removed `expectedStatus`)
- Added `author` field to all test manifests
- Added optional chaining for all `manifest.contributes` accesses

### 4. Removed Inconsistent Store Type

The `PluginContributes` interface in `src/lib/stores/plugins.ts` was a different type (counts instead of arrays). This is kept separate as it serves a different purpose (UI display metadata).

## Contract Alignment

### ✅ Synchronized Fields

| Field | Rust Type | TypeScript Type | Status |
|-------|-----------|-----------------|--------|
| `services` | `Vec<ServiceDefinition>` | `ServiceDefinition[]?` | ✅ Synced |
| `hostlists` | `Vec<HostlistDefinition>` | `HostlistDefinition[]?` | ✅ Synced |
| `strategies` | `Vec<StrategyDefinition>` | `StrategyDefinition[]?` | ✅ Synced |

### 🎨 TypeScript-Only Fields (UI Plugins)

| Field | TypeScript Type | Purpose |
|-------|-----------------|---------|
| `widgets` | `WidgetDefinition[]?` | Dashboard/sidebar widgets |
| `settings` | `SettingsPanelDefinition[]?` | Settings panels |

These fields are only used by Level 2 (UI) plugins and don't exist in Rust backend.

## Verification

### Rust Compilation
```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
✅ **Result:** Compiles successfully with 0 errors

### TypeScript Type Checking
```bash
pnpm run check
```
✅ **Result:** 0 errors, 36 warnings (all accessibility/style warnings, no type errors)

## Migration Notes

### For Plugin Developers

1. **`author` is now required** in `PluginManifest`
2. **`ServiceEndpoint` structure changed:**
   - `id` and `name` are now required
   - `expectedStatus` and `timeout` removed (not in Rust contract)
3. **`contributes` is optional** - can be omitted if plugin doesn't contribute anything
4. **Use optional chaining** when accessing `manifest.contributes?.field`

### Example Plugin Manifest

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "type": "service-checker",
  "contributes": {
    "services": [
      {
        "id": "my-service",
        "name": "My Service",
        "icon": "🎯",
        "category": "custom",
        "endpoints": [
          {
            "id": "main",
            "name": "Main Endpoint",
            "url": "https://example.com",
            "method": "GET"
          }
        ]
      }
    ]
  }
}
```

## Backward Compatibility

✅ **Maintained** - Legacy fields (`service`, `strategy`, `hostlist`) are still supported in both Rust and TypeScript for backward compatibility with existing plugins.

## Next Steps

- [ ] Update plugin documentation with new contract
- [ ] Migrate existing plugins to use `contributes` instead of legacy fields
- [ ] Add validation for plugin manifests at load time
- [ ] Consider adding JSON schema for plugin.json validation
