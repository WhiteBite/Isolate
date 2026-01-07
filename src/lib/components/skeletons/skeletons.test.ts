import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';

// Import components to verify they export correctly
import {
  ServicesSkeleton,
  ProxiesSkeleton,
  RoutingSkeleton,
  StrategiesSkeleton
} from './index';

describe('Skeleton Components', () => {
  describe('ServicesSkeleton', () => {
    it('exports correctly', () => {
      expect(ServicesSkeleton).toBeDefined();
    });

    it('contains animate-pulse classes', () => {
      const content = readFileSync(
        join(__dirname, 'ServicesSkeleton.svelte'),
        'utf-8'
      );
      expect(content).toContain('animate-pulse');
    });
  });

  describe('ProxiesSkeleton', () => {
    it('exports correctly', () => {
      expect(ProxiesSkeleton).toBeDefined();
    });

    it('contains animate-pulse classes', () => {
      const content = readFileSync(
        join(__dirname, 'ProxiesSkeleton.svelte'),
        'utf-8'
      );
      expect(content).toContain('animate-pulse');
    });
  });

  describe('RoutingSkeleton', () => {
    it('exports correctly', () => {
      expect(RoutingSkeleton).toBeDefined();
    });

    it('contains animate-pulse classes', () => {
      const content = readFileSync(
        join(__dirname, 'RoutingSkeleton.svelte'),
        'utf-8'
      );
      expect(content).toContain('animate-pulse');
    });
  });

  describe('StrategiesSkeleton', () => {
    it('exports correctly', () => {
      expect(StrategiesSkeleton).toBeDefined();
    });

    it('contains animate-pulse classes', () => {
      const content = readFileSync(
        join(__dirname, 'StrategiesSkeleton.svelte'),
        'utf-8'
      );
      expect(content).toContain('animate-pulse');
    });
  });

  describe('All skeletons', () => {
    it('all components are exported from index', () => {
      expect(ServicesSkeleton).toBeDefined();
      expect(ProxiesSkeleton).toBeDefined();
      expect(RoutingSkeleton).toBeDefined();
      expect(StrategiesSkeleton).toBeDefined();
    });

    it('all skeleton files contain proper skeleton structure', () => {
      const skeletonFiles = [
        'ServicesSkeleton.svelte',
        'ProxiesSkeleton.svelte',
        'RoutingSkeleton.svelte',
        'StrategiesSkeleton.svelte'
      ];

      for (const file of skeletonFiles) {
        const content = readFileSync(join(__dirname, file), 'utf-8');
        
        // Each skeleton should have animate-pulse for loading animation
        expect(content).toContain('animate-pulse');
        
        // Each skeleton should have proper styling classes
        expect(content).toContain('bg-zinc');
        expect(content).toContain('rounded');
      }
    });
  });
});
