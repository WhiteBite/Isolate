import type { StateMachineConfig, StateTransition, StateSnapshot } from './types';

export class StateMachine<S extends string, C extends Record<string, unknown>> {
  private config: StateMachineConfig<S, C>;
  private currentState: S;
  private context: C;
  private listeners: Set<(state: S, context: C) => void> = new Set();
  private history: StateSnapshot<S, C>[] = [];
  private maxHistory = 10;

  constructor(config: StateMachineConfig<S, C>) {
    this.config = config;
    this.currentState = config.initial;
    this.context = { ...config.initialContext };
    this.saveSnapshot();
  }

  get state(): S {
    return this.currentState;
  }

  getContext(): C {
    return { ...this.context };
  }

  getHistory(): StateSnapshot<S, C>[] {
    return [...this.history];
  }

  canTransition(event: string): boolean {
    return this.config.transitions.some(t => 
      t.event === event && 
      this.matchesFrom(t.from, this.currentState)
    );
  }

  getAvailableEvents(): string[] {
    return this.config.transitions
      .filter(t => this.matchesFrom(t.from, this.currentState))
      .map(t => t.event);
  }

  transition(event: string, contextUpdate?: Partial<C>): boolean {
    const transition = this.config.transitions.find(t =>
      t.event === event &&
      this.matchesFrom(t.from, this.currentState)
    );

    if (!transition) {
      console.warn(`Invalid transition: ${event} from ${this.currentState}`);
      return false;
    }

    const previousState = this.currentState;
    this.currentState = transition.to;
    
    if (contextUpdate) {
      this.context = { ...this.context, ...contextUpdate };
    }

    this.saveSnapshot();
    this.notify();
    
    console.debug(`State transition: ${previousState} → ${this.currentState} (${event})`);
    return true;
  }

  updateContext(update: Partial<C>): void {
    this.context = { ...this.context, ...update };
    this.notify();
  }

  subscribe(listener: (state: S, context: C) => void): () => void {
    this.listeners.add(listener);
    // Immediately call with current state
    listener(this.currentState, this.context);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach(l => l(this.currentState, this.context));
  }

  private saveSnapshot(): void {
    this.history.push({
      state: this.currentState,
      context: { ...this.context },
      timestamp: Date.now()
    });
    
    if (this.history.length > this.maxHistory) {
      this.history.shift();
    }
  }

  private matchesFrom(from: S | S[], current: S): boolean {
    if (Array.isArray(from)) {
      return from.includes(current);
    }
    return from === current;
  }

  reset(): void {
    this.currentState = this.config.initial;
    this.context = { ...this.config.initialContext };
    this.history = [];
    this.saveSnapshot();
    this.notify();
  }

  matches(state: S | S[]): boolean {
    if (Array.isArray(state)) {
      return state.includes(this.currentState);
    }
    return this.currentState === state;
  }
}
