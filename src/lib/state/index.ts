// Types
export type {
  BaseState,
  ProtectionState,
  ServiceState,
  TestState,
  StateTransition,
  StateMachineConfig,
  StateSnapshot
} from './types';

// Core StateMachine
export { StateMachine } from './stateMachine';

// Protection Machine
export {
  createProtectionMachine,
  startProtection,
  activateProtection,
  stopProtection,
  handleProtectionError,
  attemptRecovery,
  type ProtectionMachine,
  type ProtectionContext
} from './protectionMachine';

// Service Machine
export {
  createServiceMachine,
  checkService,
  markServiceAvailable,
  markServiceBlocked,
  markServiceError,
  resetService,
  ServiceMachineManager,
  type ServiceMachine,
  type ServiceContext
} from './serviceMachine';
