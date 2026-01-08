# Settings UX Model

## Обзор

Страница Settings использует **гибридную модель сохранения** для оптимального UX:
- **Auto-save** — для простых настроек с мгновенным эффектом
- **Draft + Save** — для сложных настроек, требующих подтверждения

## Модели сохранения

### 🔄 Auto-save (мгновенное сохранение)

**Когда использовать:**
- Простые toggle-переключатели
- Настройки с немедленным визуальным эффектом
- Низкорисковые изменения

**Настройки с auto-save:**

| Настройка | Вкладка | Причина |
|-----------|---------|---------|
| `autoStart` | General | Простой toggle, мгновенный эффект |
| `minimizeToTray` | General | Простой toggle |
| `notifications` | General | Простой toggle |
| `language` | General | Мгновенная смена языка UI |
| `theme` | General | Мгновенная смена темы |
| `blockQuic` | Advanced | Простой toggle |
| `debugMode` | Advanced | Простой toggle |
| `windivertMode` | Advanced | Dropdown с немедленным эффектом |
| `gameFilterMode` | Advanced | Toggle с визуальным feedback |
| `dnsOverride` | Advanced | Текстовое поле |
| `tcpTimestamps` | Advanced | Toggle (требует admin) |

**UX поведение:**
- ✅ Изменение применяется сразу при клике/изменении
- ✅ Нет toast-уведомлений (избегаем notification fatigue)
- ⚠️ Toast показывается только при ошибке
- 🔵 Визуальный индикатор "auto-save" рядом с настройкой

### 💾 Draft + Save (явное сохранение)

**Когда использовать:**
- Списки/коллекции (domain exceptions)
- Настройки с комплексным вводом (hotkeys)
- Изменения, которые пользователь может захотеть отменить
- Настройки, требующие batch-сохранения

**Настройки с draft+save:**

| Настройка | Вкладка | Причина |
|-----------|---------|---------|
| `domainExceptions` | Routing | Список доменов, batch-редактирование |
| `hotkeys` | Hotkeys | Комплексный ввод, возможность отмены |

**UX поведение:**
- ⚠️ Изменения накапливаются локально
- 🟡 Индикатор "unsaved changes" при наличии изменений
- 💾 Кнопка Save применяет все изменения
- ✅ Toast "Saved" при успешном сохранении
- ❌ Toast с ошибкой при неудаче

## Визуальные индикаторы

### Info-блоки по вкладкам

Каждая вкладка имеет info-блок в начале:

**General tab (auto-save):**
```
🔵 Settings on this page are saved automatically when changed.
   The Save button is for domain exceptions and hotkeys only.
```

**Routing tab (draft+save):**
```
🟡 Domain exceptions require clicking Save to apply.
   You can add multiple domains before saving.
```

**Hotkeys tab (draft+save):**
```
🟡 Hotkey changes require clicking Save to persist.
   Configure all your hotkeys, then save once.
```

**Advanced tab (mixed):**
```
🔵 Advanced settings are saved automatically when changed.
   Auto-Recovery has its own Save button.
```

### Индикаторы состояния

1. **Auto-save indicator** — маленькая иконка ✓ или "auto" badge
2. **Unsaved changes** — жёлтая точка или badge "Unsaved"
3. **Saving spinner** — анимация на кнопке Save
4. **Success toast** — зелёный, исчезает через 3 сек
5. **Error toast** — красный, исчезает через 5 сек

## Реализация

### Auto-save функция

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
```

### Draft+Save функция

```typescript
/**
 * Save settings that require explicit user action (Save button).
 * Used for complex settings like domain exceptions, hotkeys.
 * Shows toast notification on success/failure.
 */
async function saveSettings() {
  saving = true;
  try {
    await invoke('save_settings', { settings: localSettings });
    saveMessage = { text: 'Settings saved', type: 'success' };
  } catch (e) {
    saveMessage = { text: `Failed: ${e}`, type: 'error' };
  } finally {
    saving = false;
  }
}
```

## Компоненты с собственной логикой сохранения

Некоторые компоненты имеют собственную кнопку Save:

- **AutoRecoverySettings** — сложная конфигурация failover
- **ProviderSelector** — выбор ISP профиля

Эти компоненты инкапсулируют свою логику сохранения и не зависят от глобальной кнопки Save.

## Рекомендации для новых настроек

### Выбор модели

```
Новая настройка
    │
    ▼
┌─────────────────────────────┐
│ Это простой toggle/dropdown?│
└─────────────────────────────┘
    │ ДА                │ НЕТ
    ▼                   ▼
Auto-save          ┌──────────────────────┐
                   │ Это список/коллекция?│
                   └──────────────────────┘
                        │ ДА           │ НЕТ
                        ▼              ▼
                   Draft+Save    ┌─────────────────────┐
                                 │ Требует подтверждения│
                                 │ или может быть отменён?│
                                 └─────────────────────┘
                                      │ ДА        │ НЕТ
                                      ▼           ▼
                                 Draft+Save   Auto-save
```

### Checklist для новой настройки

- [ ] Определить модель сохранения (auto-save / draft+save)
- [ ] Добавить в соответствующую секцию UI
- [ ] Добавить комментарий в коде о модели
- [ ] Обновить info-блок вкладки если нужно
- [ ] Добавить в эту документацию
