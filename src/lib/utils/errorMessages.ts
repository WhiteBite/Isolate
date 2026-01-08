/**
 * Маппинг технических ошибок в человекочитаемые сообщения
 */

const errorPatterns: [RegExp, string][] = [
  [/Connection refused/i, 'Не удалось подключиться к серверу'],
  [/ETIMEDOUT/i, 'Превышено время ожидания'],
  [/ECONNRESET/i, 'Соединение было сброшено'],
  [/ECONNABORTED/i, 'Соединение прервано'],
  [/ENOTFOUND/i, 'Сервер не найден'],
  [/EHOSTUNREACH/i, 'Хост недоступен'],
  [/502|503|504/i, 'Сервер временно недоступен'],
  [/500/i, 'Внутренняя ошибка сервера'],
  [/401|403/i, 'Доступ запрещён'],
  [/404/i, 'Ресурс не найден'],
  [/DNS.*failed|ENOTFOUND/i, 'Ошибка DNS — не удалось разрешить адрес'],
  [/certificate|SSL|TLS/i, 'Проблема с сертификатом безопасности'],
  [/network.*error/i, 'Ошибка сети'],
  [/timeout/i, 'Превышено время ожидания'],
  [/socket hang up/i, 'Соединение неожиданно закрыто'],
  [/EPERM|EACCES/i, 'Недостаточно прав доступа'],
  [/ENOENT/i, 'Файл или путь не найден'],
  [/WinDivert/i, 'Ошибка драйвера WinDivert'],
  [/winws.*failed|winws.*error/i, 'Ошибка запуска winws'],
  [/process.*exit/i, 'Процесс завершился с ошибкой'],
];

/**
 * Преобразует техническую ошибку в понятное пользователю сообщение
 */
export function humanizeError(error: string): string {
  for (const [pattern, message] of errorPatterns) {
    if (pattern.test(error)) return message;
  }
  // Если не нашли паттерн, возвращаем оригинал, но убираем технические детали
  return error.length > 100 ? error.slice(0, 100) + '...' : error;
}

/**
 * Возвращает рекомендацию по исправлению ошибки
 */
export function getErrorSuggestion(error: string): string | null {
  if (/ETIMEDOUT|timeout/i.test(error)) {
    return 'Попробуйте другую стратегию или проверьте подключение к интернету';
  }
  if (/DNS|ENOTFOUND/i.test(error)) {
    return 'Попробуйте сменить DNS сервер в настройках системы';
  }
  if (/certificate|SSL|TLS/i.test(error)) {
    return 'Проверьте системное время или попробуйте другую стратегию';
  }
  if (/Connection refused/i.test(error)) {
    return 'Убедитесь, что сервис запущен и порт не заблокирован';
  }
  if (/WinDivert|winws/i.test(error)) {
    return 'Попробуйте перезапустить приложение от имени администратора';
  }
  if (/EPERM|EACCES/i.test(error)) {
    return 'Запустите приложение от имени администратора';
  }
  if (/502|503|504/i.test(error)) {
    return 'Подождите немного и попробуйте снова';
  }
  return null;
}

/**
 * Определяет тип ошибки для выбора иконки/цвета
 */
export type ErrorCategory = 'network' | 'permission' | 'server' | 'timeout' | 'unknown';

export function categorizeError(error: string): ErrorCategory {
  if (/ETIMEDOUT|timeout/i.test(error)) return 'timeout';
  if (/EPERM|EACCES|401|403/i.test(error)) return 'permission';
  if (/500|502|503|504/i.test(error)) return 'server';
  if (/network|DNS|connection|socket/i.test(error)) return 'network';
  return 'unknown';
}
