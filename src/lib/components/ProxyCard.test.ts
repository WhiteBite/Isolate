/**
 * Unit tests for ProxyCard component - ARIA labels and accessibility
 */

import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import ProxyCard from './ProxyCard.svelte';

afterEach(() => {
  cleanup();
});

describe('ProxyCard', () => {
  const defaultProps = {
    id: 'test-proxy',
    name: 'Test Proxy',
    server: 'example.com',
    port: 443,
    protocol: 'VLESS'
  };

  describe('rendering', () => {
    it('should render without errors', () => {
      expect(() => render(ProxyCard, { props: defaultProps })).not.toThrow();
    });

    it('should display proxy information', () => {
      render(ProxyCard, { props: defaultProps });
      expect(screen.getByText('Test Proxy')).toBeTruthy();
      expect(screen.getByText('example.com:443')).toBeTruthy();
      expect(screen.getByText('VLESS')).toBeTruthy();
    });
  });

  describe('ARIA labels - Share button', () => {
    it('should have aria-label on Share button when onShare is provided', () => {
      const onShare = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onShare } 
      });
      
      const shareButton = container.querySelector('button[title="Share QR code"]');
      expect(shareButton).toBeTruthy();
      expect(shareButton?.getAttribute('aria-label')).toBe('Share Test Proxy QR code');
    });

    it('should include proxy name in Share aria-label', () => {
      const onShare = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, name: 'My Custom Proxy', onShare } 
      });
      
      const shareButton = container.querySelector('button[title="Share QR code"]');
      expect(shareButton?.getAttribute('aria-label')).toBe('Share My Custom Proxy QR code');
    });

    it('Share aria-label should contain meaningful text', () => {
      const onShare = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onShare } 
      });
      
      const shareButton = container.querySelector('button[title="Share QR code"]');
      const ariaLabel = shareButton?.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('Share');
      expect(ariaLabel).toContain('QR code');
      expect(ariaLabel.length).toBeGreaterThan(10);
    });
  });

  describe('ARIA labels - Copy button', () => {
    it('should have aria-label on Copy button', () => {
      const { container } = render(ProxyCard, { props: defaultProps });
      
      const copyButton = container.querySelector('button[title="Copy address"]');
      expect(copyButton).toBeTruthy();
      expect(copyButton?.getAttribute('aria-label')).toBe('Copy Test Proxy address');
    });

    it('should include proxy name in Copy aria-label', () => {
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, name: 'Production Server' } 
      });
      
      const copyButton = container.querySelector('button[title="Copy address"]');
      expect(copyButton?.getAttribute('aria-label')).toBe('Copy Production Server address');
    });

    it('Copy aria-label should contain meaningful text', () => {
      const { container } = render(ProxyCard, { props: defaultProps });
      
      const copyButton = container.querySelector('button[title="Copy address"]');
      const ariaLabel = copyButton?.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('Copy');
      expect(ariaLabel).toContain('address');
      expect(ariaLabel.length).toBeGreaterThan(10);
    });
  });

  describe('ARIA labels - Edit button', () => {
    it('should have aria-label on Edit button when onEdit is provided', () => {
      const onEdit = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onEdit } 
      });
      
      const editButton = container.querySelector('button[title="Edit"]');
      expect(editButton).toBeTruthy();
      expect(editButton?.getAttribute('aria-label')).toBe('Edit Test Proxy');
    });

    it('should include proxy name in Edit aria-label', () => {
      const onEdit = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, name: 'VPN Server', onEdit } 
      });
      
      const editButton = container.querySelector('button[title="Edit"]');
      expect(editButton?.getAttribute('aria-label')).toBe('Edit VPN Server');
    });

    it('Edit aria-label should contain meaningful text', () => {
      const onEdit = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onEdit } 
      });
      
      const editButton = container.querySelector('button[title="Edit"]');
      const ariaLabel = editButton?.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('Edit');
      expect(ariaLabel.length).toBeGreaterThan(5);
    });
  });

  describe('ARIA labels - Delete button', () => {
    it('should have aria-label on Delete button when onDelete is provided', () => {
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onDelete } 
      });
      
      const deleteButton = container.querySelector('button[title="Delete"]');
      expect(deleteButton).toBeTruthy();
      expect(deleteButton?.getAttribute('aria-label')).toBe('Delete Test Proxy');
    });

    it('should include proxy name in Delete aria-label', () => {
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, name: 'Old Proxy', onDelete } 
      });
      
      const deleteButton = container.querySelector('button[title="Delete"]');
      expect(deleteButton?.getAttribute('aria-label')).toBe('Delete Old Proxy');
    });

    it('Delete aria-label should contain meaningful text', () => {
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onDelete } 
      });
      
      const deleteButton = container.querySelector('button[title="Delete"]');
      const ariaLabel = deleteButton?.getAttribute('aria-label') || '';
      expect(ariaLabel).toContain('Delete');
      expect(ariaLabel.length).toBeGreaterThan(5);
    });
  });

  describe('ARIA labels - All buttons comprehensive check', () => {
    it('should have aria-label on all action buttons', () => {
      const onShare = vi.fn();
      const onEdit = vi.fn();
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onShare, onEdit, onDelete } 
      });
      
      // Get all buttons in the action area
      const buttons = container.querySelectorAll('button');
      
      // Filter out the main card button (role="button" on div)
      const actionButtons = Array.from(buttons).filter(btn => 
        btn.getAttribute('title') !== null
      );
      
      // Should have 4 action buttons: Share, Copy, Edit, Delete
      expect(actionButtons.length).toBe(4);
      
      // All should have aria-label
      actionButtons.forEach(button => {
        const ariaLabel = button.getAttribute('aria-label');
        expect(ariaLabel).toBeTruthy();
        expect(ariaLabel!.length).toBeGreaterThan(0);
      });
    });

    it('all aria-labels should be unique and descriptive', () => {
      const onShare = vi.fn();
      const onEdit = vi.fn();
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onShare, onEdit, onDelete } 
      });
      
      const buttons = container.querySelectorAll('button[aria-label]');
      const ariaLabels = Array.from(buttons).map(btn => btn.getAttribute('aria-label'));
      
      // All labels should be unique
      const uniqueLabels = new Set(ariaLabels);
      expect(uniqueLabels.size).toBe(ariaLabels.length);
      
      // All labels should contain the proxy name
      ariaLabels.forEach(label => {
        expect(label).toContain('Test Proxy');
      });
    });
  });

  describe('button functionality', () => {
    it('should call onShare when Share button is clicked', async () => {
      const onShare = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onShare } 
      });
      
      const shareButton = container.querySelector('button[aria-label*="Share"]');
      await fireEvent.click(shareButton!);
      expect(onShare).toHaveBeenCalledTimes(1);
    });

    it('should call onEdit when Edit button is clicked', async () => {
      const onEdit = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onEdit } 
      });
      
      const editButton = container.querySelector('button[aria-label*="Edit"]');
      await fireEvent.click(editButton!);
      expect(onEdit).toHaveBeenCalledTimes(1);
    });

    it('should call onDelete when Delete button is clicked', async () => {
      const onDelete = vi.fn();
      const { container } = render(ProxyCard, { 
        props: { ...defaultProps, onDelete } 
      });
      
      const deleteButton = container.querySelector('button[aria-label*="Delete"]');
      await fireEvent.click(deleteButton!);
      expect(onDelete).toHaveBeenCalledTimes(1);
    });
  });
});
