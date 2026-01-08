import { StateMachine } from './stateMachine';
import type { ServiceState, StateMachineConfig } from './types';

export interface ServiceContext extends Record<string, unknown> {
  serviceId: string;
  lastCheck: number | null;
  latency: number | null;
  errorCount: number;
  lastError: string | null;
  consecutiveFailures: number;
}

const serviceConfig: StateMachineConfig<ServiceState, ServiceContext> = {
  initial: 'unknown',
  initialContext: {
    serviceId: '',
    lastCheck: null,
    latency: null,
    errorCount: 0,
    lastError: null,
    consecutiveFailures: 0
  },
  transitions: [
    // Check flow
    { from: 'unknown', to: 'checking', event: 'CHECK' },
    { from: 'available', to: 'checking', event: 'CHECK' },
    { from: 'blocked', to: 'checking', event: 'CHECK' },
    { from: 'error', to: 'checking', event: 'CHECK' },
    
    // Results
    { from: 'checking', to: 'available', event: 'AVAILABLE' },
    { from: 'checking', to: 'blocked', event: 'BLOCKED' },
    { from: 'checking', to: 'error', event: 'ERROR' },
    
    // Reset
    { from: ['unknown', 'checking', 'available', 'blocked', 'error'], to: 'unknown', event: 'RESET' }
  ]
};

export function createServiceMachine(serviceId: string): StateMachine<ServiceState, ServiceContext> {
  const config = {
    ...serviceConfig,
    initialContext: {
      ...serviceConfig.initialContext,
      serviceId
    }
  };
  return new StateMachine(config);
}

export type ServiceMachine = ReturnType<typeof createServiceMachine>;

// Helper functions
export function checkService(machine: ServiceMachine): boolean {
  return machine.transition('CHECK', {
    lastCheck: Date.now()
  });
}

export function markServiceAvailable(
  machine: ServiceMachine, 
  latency: number
): boolean {
  return machine.transition('AVAILABLE', {
    latency,
    lastCheck: Date.now(),
    consecutiveFailures: 0,
    lastError: null
  });
}

export function markServiceBlocked(machine: ServiceMachine): boolean {
  const context = machine.getContext();
  return machine.transition('BLOCKED', {
    latency: null,
    lastCheck: Date.now(),
    consecutiveFailures: context.consecutiveFailures + 1
  });
}

export function markServiceError(
  machine: ServiceMachine, 
  error: string
): boolean {
  const context = machine.getContext();
  return machine.transition('ERROR', {
    latency: null,
    lastCheck: Date.now(),
    errorCount: context.errorCount + 1,
    consecutiveFailures: context.consecutiveFailures + 1,
    lastError: error
  });
}

export function resetService(machine: ServiceMachine): boolean {
  return machine.transition('RESET');
}

// Batch service management
export class ServiceMachineManager {
  private machines: Map<string, ServiceMachine> = new Map();

  getOrCreate(serviceId: string): ServiceMachine {
    let machine = this.machines.get(serviceId);
    if (!machine) {
      machine = createServiceMachine(serviceId);
      this.machines.set(serviceId, machine);
    }
    return machine;
  }

  get(serviceId: string): ServiceMachine | undefined {
    return this.machines.get(serviceId);
  }

  getAll(): ServiceMachine[] {
    return Array.from(this.machines.values());
  }

  getAllStates(): Map<string, ServiceState> {
    const states = new Map<string, ServiceState>();
    this.machines.forEach((machine, id) => {
      states.set(id, machine.state);
    });
    return states;
  }

  checkAll(): void {
    this.machines.forEach(machine => checkService(machine));
  }

  resetAll(): void {
    this.machines.forEach(machine => resetService(machine));
  }

  remove(serviceId: string): boolean {
    return this.machines.delete(serviceId);
  }

  clear(): void {
    this.machines.clear();
  }
}
