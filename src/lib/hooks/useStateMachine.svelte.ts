import { StateMachine } from '$lib/state/stateMachine';

/**
 * Svelte 5 hook for using StateMachine with runes
 * Provides reactive state and context that update automatically
 */
export function useStateMachine<S extends string, C extends Record<string, unknown>>(
  machine: StateMachine<S, C>
) {
  let state = $state(machine.state);
  let context = $state(machine.getContext());

  $effect(() => {
    const unsubscribe = machine.subscribe((newState, newContext) => {
      state = newState;
      context = newContext;
    });
    return unsubscribe;
  });

  function send(event: string, contextUpdate?: Partial<C>): boolean {
    return machine.transition(event, contextUpdate);
  }

  function can(event: string): boolean {
    return machine.canTransition(event);
  }

  function matches(stateOrStates: S | S[]): boolean {
    return machine.matches(stateOrStates);
  }

  return {
    get state() { return state; },
    get context() { return context; },
    send,
    can,
    matches,
    reset: () => machine.reset(),
    getAvailableEvents: () => machine.getAvailableEvents(),
    getHistory: () => machine.getHistory()
  };
}

/**
 * Creates a new StateMachine and returns the hook
 * Convenience function for one-liner usage
 */
export function createStateMachineHook<S extends string, C extends Record<string, unknown>>(
  config: import('$lib/state/types').StateMachineConfig<S, C>
) {
  const machine = new StateMachine(config);
  return useStateMachine(machine);
}
