/** Available animation variants */
export type TransitionVariant = 'fade' | 'slide' | 'scale' | 'blur' | 'slide-up' | 'slide-down' | 'zoom';

/** Stagger configuration for child elements */
export interface StaggerConfig {
  /** Enable staggered animation for children */
  enabled: boolean;
  /** Delay between each child in ms */
  delay?: number;
  /** CSS selector for children to animate */
  selector?: string;
}
