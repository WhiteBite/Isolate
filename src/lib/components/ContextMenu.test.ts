/**
 * Unit tests for ContextMenu, ContextMenuItem, ContextMenuSeparator components
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import ContextMenu from './ContextMenu.svelte';
import ContextMenuItem from './ContextMenuItem.svelte';
import ContextMenuSeparator from './ContextMenuSeparator.svelte';

beforeEach(() => {
  Object.defineProperty(window, 'innerWidth', { value: 1920, writable: true });
  Object.defineProperty(window, 'innerHeight', { value: 1080, writable: true });
  vi.spyOn(window, 'requestAnimationFrame').mockImplementation((cb) => {
    cb(0);
    return 0;
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function createMouseEvent(x: number, y: number): MouseEvent {
  return new MouseEvent('contextmenu', {
    clientX: x,
    clientY: y,
    bubbles: true,
    cancelable: true
  });
}

describe('ContextMenu', () => {
  describe('rendering', () => {
    it('should not render menu content when not visible', () => {
      render(ContextMenu);
      const menu = screen.queryByRole('menu');
      expect(menu).toBeNull();
    });

    it('should render menu with role="menu" when shown', async () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(100, 200);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        const menu = screen.getByRole('menu');
        expect(menu).toBeTruthy();
      });
    });

    it('should position menu at mouse coordinates', async () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(150, 250);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        const menu = screen.getByRole('menu');
        expect(menu.style.left).toBe('150px');
        expect(menu.style.top).toBe('250px');
      });
    });
  });

  describe('show() method', () => {
    it('should prevent default event behavior', () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(100, 100);
      const preventDefaultSpy = vi.spyOn(mockEvent, 'preventDefault');
      const stopPropagationSpy = vi.spyOn(mockEvent, 'stopPropagation');
      component.show(mockEvent);
      expect(preventDefaultSpy).toHaveBeenCalled();
      expect(stopPropagationSpy).toHaveBeenCalled();
    });

    it('should make menu visible after calling show()', async () => {
      const { component } = render(ContextMenu);
      expect(screen.queryByRole('menu')).toBeNull();
      const mockEvent = createMouseEvent(100, 100);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        expect(screen.getByRole('menu')).toBeTruthy();
      });
    });
  });

  describe('hide() method', () => {
    it('should hide menu when hide() is called', async () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(100, 100);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        expect(screen.getByRole('menu')).toBeTruthy();
      });
      component.hide();
      await tick();
      await waitFor(() => {
        expect(screen.queryByRole('menu')).toBeNull();
      });
    });
  });

  describe('keyboard interaction', () => {
    it('should hide menu on Escape key', async () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(100, 100);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        expect(screen.getByRole('menu')).toBeTruthy();
      });
      await fireEvent.keyDown(window, { key: 'Escape' });
      await tick();
      await waitFor(() => {
        expect(screen.queryByRole('menu')).toBeNull();
      });
    });
  });

  describe('click outside', () => {
    it('should hide menu when clicking outside', async () => {
      const { component } = render(ContextMenu);
      const mockEvent = createMouseEvent(100, 100);
      component.show(mockEvent);
      await tick();
      await waitFor(() => {
        expect(screen.getByRole('menu')).toBeTruthy();
      });
      await fireEvent.click(document.body);
      await tick();
      await waitFor(() => {
        expect(screen.queryByRole('menu')).toBeNull();
      });
    });
  });
});

describe('ContextMenuItem', () => {
  describe('rendering', () => {
    it('should render with role="menuitem"', () => {
      render(ContextMenuItem);
      const item = screen.getByRole('menuitem');
      expect(item).toBeTruthy();
    });

    it('should render icon when provided', () => {
      render(ContextMenuItem, { props: { icon: '🔄' } });
      const item = screen.getByRole('menuitem');
      expect(item.textContent).toContain('🔄');
    });

    it('should render shortcut when provided', () => {
      render(ContextMenuItem, { props: { shortcut: 'Ctrl+C' } });
      const item = screen.getByRole('menuitem');
      expect(item.textContent).toContain('Ctrl+C');
    });

    it('should apply danger variant styles', () => {
      render(ContextMenuItem, { props: { variant: 'danger' } });
      const item = screen.getByRole('menuitem');
      expect(item.className).toContain('text-neon-red');
    });

    it('should apply default variant styles', () => {
      render(ContextMenuItem, { props: { variant: 'default' } });
      const item = screen.getByRole('menuitem');
      expect(item.className).toContain('text-white/90');
    });
  });

  describe('disabled state', () => {
    it('should be disabled when disabled prop is true', () => {
      render(ContextMenuItem, { props: { disabled: true } });
      const item = screen.getByRole('menuitem');
      expect(item).toHaveProperty('disabled', true);
    });

    it('should have tabindex=-1 when disabled', () => {
      render(ContextMenuItem, { props: { disabled: true } });
      const item = screen.getByRole('menuitem');
      expect(item.getAttribute('tabindex')).toBe('-1');
    });

    it('should have tabindex=0 when not disabled', () => {
      render(ContextMenuItem, { props: { disabled: false } });
      const item = screen.getByRole('menuitem');
      expect(item.getAttribute('tabindex')).toBe('0');
    });

    it('should apply opacity styles when disabled', () => {
      render(ContextMenuItem, { props: { disabled: true } });
      const item = screen.getByRole('menuitem');
      expect(item.className).toContain('opacity-40');
    });
  });

  describe('click handling', () => {
    it('should call onclick when clicked', async () => {
      const handleClick = vi.fn();
      render(ContextMenuItem, { props: { onclick: handleClick } });
      const item = screen.getByRole('menuitem');
      await fireEvent.click(item);
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it('should NOT call onclick when disabled', async () => {
      const handleClick = vi.fn();
      render(ContextMenuItem, { props: { onclick: handleClick, disabled: true } });
      const item = screen.getByRole('menuitem');
      await fireEvent.click(item);
      expect(handleClick).not.toHaveBeenCalled();
    });

    it('should call onclick on Enter key', async () => {
      const handleClick = vi.fn();
      render(ContextMenuItem, { props: { onclick: handleClick } });
      const item = screen.getByRole('menuitem');
      await fireEvent.keyDown(item, { key: 'Enter' });
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it('should call onclick on Space key', async () => {
      const handleClick = vi.fn();
      render(ContextMenuItem, { props: { onclick: handleClick } });
      const item = screen.getByRole('menuitem');
      await fireEvent.keyDown(item, { key: ' ' });
      expect(handleClick).toHaveBeenCalledTimes(1);
    });

    it('should NOT call onclick on other keys', async () => {
      const handleClick = vi.fn();
      render(ContextMenuItem, { props: { onclick: handleClick } });
      const item = screen.getByRole('menuitem');
      await fireEvent.keyDown(item, { key: 'Tab' });
      expect(handleClick).not.toHaveBeenCalled();
    });
  });
});

describe('ContextMenuSeparator', () => {
  describe('rendering', () => {
    it('should render with role="separator"', () => {
      render(ContextMenuSeparator);
      const separator = screen.getByRole('separator');
      expect(separator).toBeTruthy();
    });

    it('should have correct styling class', () => {
      render(ContextMenuSeparator);
      const separator = screen.getByRole('separator');
      expect(separator.className).toContain('h-px');
      expect(separator.className).toContain('bg-glass-border-active');
    });
  });
});
