/**
 * Unit tests for HealthWidget component
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import HealthWidget from './HealthWidget.svelte';

afterEach(() => {
  cleanup();
});

describe('HealthWidget', () => {
  describe('rendering', () => {
    it('should render without errors', () => {
      expect(() => render(HealthWidget)).not.toThrow();
    });

    it('should render default services when no props provided', () => {
      render(HealthWidget);
      expect(screen.getByText('YouTube')).toBeTruthy();
      expect(screen.getByText('Discord')).toBeTruthy();
      expect(screen.getByText('Telegram')).toBeTruthy();
      expect(screen.getByText('Twitter/X')).toBeTruthy();
    });

    it('should render custom services when provided', () => {
      const customServices = [
        { name: 'Custom Service', status: 'healthy' as const, ping: 25 }
      ];
      render(HealthWidget, { props: { services: customServices } });
      expect(screen.getByText('Custom Service')).toBeTruthy();
      expect(screen.queryByText('YouTube')).toBeNull();
    });
  });

  describe('syntax validation', () => {
    it('should not have syntax errors in status text rendering', () => {
      const { container } = render(HealthWidget);
      // Check that status labels are rendered correctly without extra characters
      const statusElements = container.querySelectorAll('.text-\\[10px\\]');
      statusElements.forEach(el => {
        const text = el.textContent?.trim() || '';
        // Should be OK, Slow, or Down - no extra > character
        expect(['OK', 'Slow', 'Down']).toContain(text);
        expect(text).not.toContain('>');
      });
    });

    it('should render status labels correctly for all service states', () => {
      const services = [
        { name: 'Service1', status: 'healthy' as const, ping: 30 },
        { name: 'Service2', status: 'degraded' as const, ping: 150 },
        { name: 'Service3', status: 'down' as const, ping: undefined }
      ];
      render(HealthWidget, { props: { services } });
      expect(screen.getByText('OK')).toBeTruthy();
      expect(screen.getByText('Slow')).toBeTruthy();
      expect(screen.getByText('Down')).toBeTruthy();
    });
  });

  describe('status indicators', () => {
    it('should render healthy status with green indicator', () => {
      const services = [{ name: 'Test', status: 'healthy' as const, ping: 40 }];
      const { container } = render(HealthWidget, { props: { services } });
      const indicator = container.querySelector('.bg-emerald-500');
      expect(indicator).toBeTruthy();
    });

    it('should render degraded status with amber indicator and pulse', () => {
      const services = [{ name: 'Test', status: 'degraded' as const, ping: 120 }];
      const { container } = render(HealthWidget, { props: { services } });
      const indicator = container.querySelector('.bg-amber-500');
      expect(indicator).toBeTruthy();
      expect(indicator?.className).toContain('animate-pulse');
    });

    it('should render down status with red indicator', () => {
      const services = [{ name: 'Test', status: 'down' as const, ping: undefined }];
      const { container } = render(HealthWidget, { props: { services } });
      const indicator = container.querySelector('.bg-red-500');
      expect(indicator).toBeTruthy();
    });
  });

  describe('ping display', () => {
    it('should display ping value when available', () => {
      const services = [{ name: 'Test', status: 'healthy' as const, ping: 45 }];
      render(HealthWidget, { props: { services } });
      expect(screen.getByText('45ms')).toBeTruthy();
    });

    it('should display placeholder when ping is undefined', () => {
      const services = [{ name: 'Test', status: 'down' as const, ping: undefined }];
      render(HealthWidget, { props: { services } });
      expect(screen.getByText('--')).toBeTruthy();
    });
  });
});
