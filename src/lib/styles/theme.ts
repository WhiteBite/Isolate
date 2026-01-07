/**
 * Centralized color system for Isolate
 * Based on Glass & Void design language
 * 
 * Custom brand colors:
 * - Cyan: #00d4ff (primary accent)
 * - Green: #00ff88 (success/active)
 * - Red: #ff3333 (error)
 * - Yellow/Amber: #ffaa00 (warning)
 * - Gray: #a0a0a0 (inactive/neutral)
 */

// Status colors used across components
export const statusColors = {
  active: {
    bg: 'bg-[#00ff88]/10',
    text: 'text-[#00ff88]',
    border: 'border-[#00ff88]/30',
    dot: 'bg-[#00ff88]'
  },
  inactive: {
    bg: 'bg-[#a0a0a0]/10',
    text: 'text-[#a0a0a0]',
    border: 'border-[#a0a0a0]/30',
    dot: 'bg-[#a0a0a0]'
  },
  error: {
    bg: 'bg-[#ff3333]/10',
    text: 'text-[#ff3333]',
    border: 'border-[#ff3333]/30',
    dot: 'bg-[#ff3333]'
  },
  warning: {
    bg: 'bg-[#ffaa00]/10',
    text: 'text-[#ffaa00]',
    border: 'border-[#ffaa00]/30',
    dot: 'bg-[#ffaa00]'
  },
  info: {
    bg: 'bg-[#00d4ff]/10',
    text: 'text-[#00d4ff]',
    border: 'border-[#00d4ff]/30',
    dot: 'bg-[#00d4ff]'
  }
} as const;

// Badge/Tag variants (combines bg, text, border)
export const badgeVariants = {
  active: 'bg-[#00ff88]/10 text-[#00ff88] border-[#00ff88]/30',
  inactive: 'bg-[#a0a0a0]/10 text-[#a0a0a0] border-[#a0a0a0]/30',
  error: 'bg-[#ff3333]/10 text-[#ff3333] border-[#ff3333]/30',
  warning: 'bg-[#ffaa00]/10 text-[#ffaa00] border-[#ffaa00]/30',
  info: 'bg-[#00d4ff]/10 text-[#00d4ff] border-[#00d4ff]/30'
} as const;

// Badge dot colors
export const badgeDotColors = {
  active: 'bg-[#00ff88]',
  inactive: 'bg-[#a0a0a0]',
  error: 'bg-[#ff3333]',
  warning: 'bg-[#ffaa00]',
  info: 'bg-[#00d4ff]'
} as const;

// Progress bar colors
export const progressColors = {
  cyan: 'bg-[#00d4ff]',
  green: 'bg-[#00ff88]',
  red: 'bg-[#ff3333]',
  yellow: 'bg-[#ffaa00]'
} as const;

// Spinner colors (using text-* for currentColor inheritance)
export const spinnerColors = {
  cyan: 'text-[#00d4ff]',
  white: 'text-white',
  gray: 'text-[#a0a0a0]'
} as const;

// Type exports
export type StatusType = keyof typeof statusColors;
export type BadgeVariant = keyof typeof badgeVariants;
export type ProgressColor = keyof typeof progressColors;
export type SpinnerColor = keyof typeof spinnerColors;
