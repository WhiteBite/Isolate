/**
 * Hook for subscribing to EventBus events with automatic cleanup
 * Uses Svelte 5 $effect for lifecycle management
 */
import { eventBus, type EventType, type EventPayload } from '$lib/stores/eventBus.svelte';

/**
 * Subscribe to an event with automatic cleanup on component destroy
 * 
 * @example
 * ```svelte
 * <script lang="ts">
 *   import { useEvent } from '$lib/hooks/useEvent.svelte';
 *   
 *   let traffic = $state({ download: 0, upload: 0 });
 *   
 *   useEvent('traffic:update', (payload) => {
 *     traffic = payload;
 *   });
 * </script>
 * ```
 * 
 * @param event - Event type to subscribe to
 * @param handler - Callback function to execute when event is emitted
 */
export function useEvent<T extends EventType>(
  event: T,
  handler: (payload: EventPayload[T]) => void
): void {
  $effect(() => {
    const unsubscribe = eventBus.on(event, handler);
    
    // Cleanup on effect re-run or component destroy
    return unsubscribe;
  });
}

/**
 * Subscribe to an event once with automatic cleanup
 * The handler will be called at most once, then automatically unsubscribed
 * 
 * @example
 * ```svelte
 * <script lang="ts">
 *   import { useEventOnce } from '$lib/hooks/useEvent.svelte';
 *   
 *   useEventOnce('strategy:changed', (payload) => {
 *     console.log('Strategy changed to:', payload.strategyId);
 *   });
 * </script>
 * ```
 * 
 * @param event - Event type to subscribe to
 * @param handler - Callback function to execute once
 */
export function useEventOnce<T extends EventType>(
  event: T,
  handler: (payload: EventPayload[T]) => void
): void {
  $effect(() => {
    const unsubscribe = eventBus.once(event, handler);
    
    // Cleanup on effect re-run or component destroy
    return unsubscribe;
  });
}

/**
 * Create an event emitter function for a specific event type
 * Useful for components that need to emit events
 * 
 * @example
 * ```svelte
 * <script lang="ts">
 *   import { useEventEmitter } from '$lib/hooks/useEvent.svelte';
 *   
 *   const emitTraffic = useEventEmitter('traffic:update');
 *   
 *   function updateTraffic() {
 *     emitTraffic({ download: 100, upload: 50 });
 *   }
 * </script>
 * ```
 * 
 * @param event - Event type to create emitter for
 * @returns Function to emit the event with payload
 */
export function useEventEmitter<T extends EventType>(
  event: T
): (payload: EventPayload[T]) => void {
  return (payload: EventPayload[T]) => {
    eventBus.emit(event, payload);
  };
}

/**
 * Subscribe to multiple events with a single handler
 * Useful when the same action should be taken for multiple events
 * 
 * @example
 * ```svelte
 * <script lang="ts">
 *   import { useEvents } from '$lib/hooks/useEvent.svelte';
 *   
 *   useEvents(['connection:opened', 'connection:closed'], (event, payload) => {
 *     console.log(`Connection event: ${event}`, payload);
 *   });
 * </script>
 * ```
 * 
 * @param events - Array of event types to subscribe to
 * @param handler - Callback function receiving event type and payload
 */
export function useEvents<T extends EventType>(
  events: T[],
  handler: (event: T, payload: EventPayload[T]) => void
): void {
  $effect(() => {
    const unsubscribes = events.map((event) =>
      eventBus.on(event, (payload) => handler(event, payload))
    );
    
    // Cleanup all subscriptions
    return () => {
      unsubscribes.forEach((unsub) => unsub());
    };
  });
}
