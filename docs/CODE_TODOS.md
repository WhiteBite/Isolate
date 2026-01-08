# TODO/FIXME из кода

> Автоматически собранный список TODO/FIXME комментариев из исходного кода.
> Последнее обновление: автоматически

---

## Frontend (Svelte/TypeScript)

### src/routes/
- [ ] `+page.svelte:370` — Replace with real network stats from backend when available

### src/lib/stores/
- [ ] `aiPilot.svelte.ts:104` — Реальная проверка через backend
- [ ] `aiPilot.svelte.ts:148` — Откатить изменение через backend
- [ ] `dashboard.ts:63` — Replace with real API calls when backend is ready
- [ ] `gameMode.svelte.ts:170` — Implement actual process detection via Tauri
- [ ] `library.svelte.ts:84` — Replace with real API
- [ ] `library.svelte.ts:192` — Real check via backend

### src/lib/components/
- [ ] `settings/AutoRecoverySettings.svelte:51` — track changes (hasChanges)

---

## Backend (Rust)

### src-tauri/src/commands/
- [ ] `troubleshoot.rs:224` — Сохранить привязку service -> strategy в настройках

---

## Статистика

| Категория | Количество |
|-----------|------------|
| Frontend (Svelte) | 2 |
| Frontend (TypeScript) | 6 |
| Backend (Rust) | 1 |
| **Всего** | **9** |

---

## Приоритеты

### Высокий приоритет (блокирует функциональность)
- `dashboard.ts:63` — Dashboard использует mock данные
- `library.svelte.ts:84` — Library использует mock данные
- `library.svelte.ts:192` — Проверка сервисов не работает

### Средний приоритет (улучшение UX)
- `+page.svelte:370` — Реальная статистика сети
- `gameMode.svelte.ts:170` — Автодетект игр
- `aiPilot.svelte.ts:104` — AI Pilot проверка
- `aiPilot.svelte.ts:148` — AI Pilot откат

### Низкий приоритет (косметические)
- `AutoRecoverySettings.svelte:51` — Отслеживание изменений
- `troubleshoot.rs:224` — Сохранение привязок
