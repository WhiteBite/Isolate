# Settings UX Model

## Current Behavior

Settings in Isolate use a **hybrid save model**:

### Auto-Save Settings (immediate)
These settings apply immediately when changed:
- Language / Theme
- Autostart on boot (`autoStart`)
- Minimize to tray (`minimizeToTray`)
- Notifications toggle
- Silent mode
- Telemetry preferences
- Block QUIC (`blockQuic`)
- Debug mode (`debugMode`)
- WinDivert mode selection
- Game filter mode
- DNS override
- TCP Timestamps

### Manual Save Settings (require confirmation)
These settings require explicit "Save" action:
- Domain exceptions (Routing tab)
- Hotkey configurations (Hotkeys tab)
- Auto-Recovery settings (has own Save button)
- ISP Profile selection (ProviderSelector)

## Implementation

### Auto-Save Pattern
```typescript
/**
 * Auto-save individual setting to backend immediately.
 * Used for simple, non-critical settings that users expect immediate effect.
 * No toast notification to avoid notification fatigue.
 */
async function setSetting(key: string, value: unknown) {
  if (!browser || !isTauri) return;
  
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_setting', { key, value });
  } catch (e) {
    console.error('Failed to set setting:', e);
    // Show error toast only on failure
    saveMessage = { text: `Failed to save: ${e}`, type: 'error' };
  }
}

// Usage example
async function handleToggle(key: string, value: boolean) {
  await setSetting(key, value);
  // No success toast - immediate visual feedback is enough
}
```

### Manual Save Pattern
```typescript
/**
 * Save settings that require explicit user action (Save button).
 * Used for complex settings like domain exceptions, hotkeys.
 * Shows toast notification on success/failure.
 */
let draft = $state({ ...currentSettings });
let hasChanges = $derived(JSON.stringify(draft) !== JSON.stringify(currentSettings));

async function saveSettings() {
  saving = true;
  try {
    await invoke('save_settings', { settings: draft });
    currentSettings = { ...draft };
    saveMessage = { text: 'Settings saved', type: 'success' };
  } catch (e) {
    saveMessage = { text: `Failed: ${e}`, type: 'error' };
  } finally {
    saving = false;
  }
}
```

## Feedback

- **Auto-save**: No toast on success (avoids notification fatigue), error toast only on failure
- **Manual save**: "Save" button enabled only when `hasChanges` is true
- **Success**: Green toast, disappears after 3 seconds
- **Errors**: Red toast with error message, disappears after 5 seconds
- **Saving state**: Spinner animation on Save button

## Visual Indicators

### Info Blocks per Tab

Each tab has an info block explaining the save behavior:

**General tab (auto-save):**
```
🔵 Settings on this page are saved automatically when changed.
```

**Routing tab (draft+save):**
```
🟡 Domain exceptions require clicking Save to apply.
```

**Hotkeys tab (draft+save):**
```
🟡 Hotkey changes require clicking Save to persist.
```

**Advanced tab (mixed):**
```
🔵 Advanced settings are saved automatically when changed.
```

### State Indicators

| Indicator | Meaning |
|-----------|---------|
| ✓ or "auto" badge | Setting uses auto-save |
| Yellow dot / "Unsaved" badge | Pending changes need Save |
| Spinner on button | Save in progress |
| Green toast | Success |
| Red toast | Error |

## Recommendations

1. **Prefer auto-save** for simple toggles and selections
2. **Use manual save** for complex/dangerous settings or collections (lists)
3. **Always show feedback** — user should know if change was applied (visual state change for auto-save, toast for manual save)
4. **Confirm destructive changes** — reset to defaults, clear data
5. **Avoid notification fatigue** — don't show success toasts for every auto-save

## Decision Tree for New Settings

```
New setting
    │
    ▼
┌─────────────────────────────┐
│ Is it a simple toggle/dropdown?│
└─────────────────────────────┘
    │ YES               │ NO
    ▼                   ▼
Auto-save          ┌──────────────────────┐
                   │ Is it a list/collection?│
                   └──────────────────────┘
                        │ YES          │ NO
                        ▼              ▼
                   Manual Save    ┌─────────────────────┐
                                  │ Requires confirmation│
                                  │ or can be cancelled? │
                                  └─────────────────────┘
                                       │ YES       │ NO
                                       ▼           ▼
                                  Manual Save   Auto-save
```

## Checklist for Adding New Settings

- [ ] Determine save model (auto-save / manual save)
- [ ] Add to appropriate UI section
- [ ] Add code comment about save model
- [ ] Update tab info block if needed
- [ ] Update this documentation
