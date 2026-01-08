export type BaseState = 'idle' | 'loading' | 'active' | 'error' | 'recovering';

export type ProtectionState = 
  | 'idle'
  | 'checking'
  | 'starting'
  | 'active'
  | 'degraded'
  | 'recovering'
  | 'stopping'
  | 'error';

export type ServiceState = 'unknown' | 'checking' | 'available' | 'blocked' | 'error';

export type TestState = 'idle' | 'preparing' | 'running' | 'analyzing' | 'complete' | 'error';

export interface StateTransition<S extends string> {
  from: S | S[];
  to: S;
  event: string;
}

export interface StateMachineConfig<S extends string, C> {
  initial: S;
  initialContext: C;
  transitions: StateTransition<S>[];
}

export interface StateSnapshot<S extends string, C> {
  state: S;
  context: C;
  timestamp: number;
}
