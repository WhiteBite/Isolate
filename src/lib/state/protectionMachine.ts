import { StateMachine } from './stateMachine';
import type { ProtectionState, StateMachineConfig } from './types';

export interface ProtectionContext extends Record<string, unknown> {
  currentStrategy: string | null;
  lastError: string | null;
  recoveryAttempts: number;
  startedAt: number | null;
  lastStateChange: number;
}

const protectionConfig: StateMachineConfig<ProtectionState, ProtectionContext> = {
  initial: 'idle',
  initialContext: {
    currentStrategy: null,
    lastError: null,
    recoveryAttempts: 0,
    startedAt: null,
    lastStateChange: Date.now()
  },
  transitions: [
    // Normal flow
    { from: 'idle', to: 'checking', event: 'CHECK' },
    { from: 'checking', to: 'starting', event: 'START' },
    { from: 'starting', to: 'active', event: 'STARTED' },
    
    // Degradation and recovery
    { from: 'active', to: 'degraded', event: 'DEGRADE' },
    { from: 'degraded', to: 'recovering', event: 'RECOVER' },
    { from: 'recovering', to: 'active', event: 'RECOVERED' },
    { from: 'recovering', to: 'degraded', event: 'RECOVER_FAILED' },
    
    // Stopping
    { from: 'active', to: 'stopping', event: 'STOP' },
    { from: 'degraded', to: 'stopping', event: 'STOP' },
    { from: 'recovering', to: 'stopping', event: 'STOP' },
    { from: 'stopping', to: 'idle', event: 'STOPPED' },
    
    // Error handling (from any state)
    { from: ['idle', 'checking', 'starting', 'active', 'degraded', 'recovering', 'stopping'], to: 'error', event: 'ERROR' },
    
    // Reset (from any state)
    { from: ['idle', 'checking', 'starting', 'active', 'degraded', 'recovering', 'stopping', 'error'], to: 'idle', event: 'RESET' },
    
    // Retry from error
    { from: 'error', to: 'checking', event: 'RETRY' }
  ]
};

export function createProtectionMachine(): StateMachine<ProtectionState, ProtectionContext> {
  return new StateMachine(protectionConfig);
}

export type ProtectionMachine = ReturnType<typeof createProtectionMachine>;

// Helper functions for common operations
export function startProtection(
  machine: ProtectionMachine, 
  strategyId: string
): boolean {
  if (!machine.canTransition('CHECK')) {
    return false;
  }
  
  machine.transition('CHECK', { 
    currentStrategy: strategyId,
    lastError: null,
    recoveryAttempts: 0,
    lastStateChange: Date.now()
  });
  
  return true;
}

export function activateProtection(machine: ProtectionMachine): boolean {
  if (machine.state === 'checking') {
    machine.transition('START');
  }
  
  if (machine.state === 'starting') {
    return machine.transition('STARTED', {
      startedAt: Date.now(),
      lastStateChange: Date.now()
    });
  }
  
  return false;
}

export function stopProtection(machine: ProtectionMachine): boolean {
  if (machine.canTransition('STOP')) {
    machine.transition('STOP', { lastStateChange: Date.now() });
    return machine.transition('STOPPED', {
      currentStrategy: null,
      startedAt: null,
      lastStateChange: Date.now()
    });
  }
  return false;
}

export function handleProtectionError(
  machine: ProtectionMachine, 
  error: string
): boolean {
  return machine.transition('ERROR', {
    lastError: error,
    lastStateChange: Date.now()
  });
}

export function attemptRecovery(machine: ProtectionMachine): boolean {
  const context = machine.getContext();
  
  if (machine.state === 'degraded') {
    return machine.transition('RECOVER', {
      recoveryAttempts: context.recoveryAttempts + 1,
      lastStateChange: Date.now()
    });
  }
  
  return false;
}
