/**
 * Navigation Store - Svelte 5 runes
 * Manages sidebar navigation state, groups, and active route
 */

export interface NavItem {
  id: string;
  name: string;
  icon: string;
  route: string;
  badge?: number;
}

export interface NavGroup {
  id: string;
  label: string;
  items: NavItem[];
}

class NavigationStore {
  collapsed = $state(false);
  activeRoute = $state('/');
  
  groups = $state<NavGroup[]>([
    {
      id: 'main',
      label: 'Main',
      items: [
        { id: 'dashboard', name: 'Dashboard', icon: 'layout-dashboard', route: '/' },
        { id: 'library', name: 'Library', icon: 'library', route: '/library' },
      ]
    },
    {
      id: 'tools',
      label: 'Tools',
      items: [
        { id: 'troubleshoot', name: 'Диагностика', icon: 'wrench', route: '/troubleshoot' },
        { id: 'optimize', name: 'Оптимизация', icon: 'wand', route: '/orchestra' },
        { id: 'proxy', name: 'Proxy & VPN', icon: 'globe', route: '/proxies' },
      ]
    },
    {
      id: 'system',
      label: 'System',
      items: [
        { id: 'plugins', name: 'Plugins', icon: 'puzzle', route: '/plugins' },
        { id: 'settings', name: 'Settings', icon: 'settings', route: '/settings' },
      ]
    }
  ]);

  toggle() {
    this.collapsed = !this.collapsed;
  }

  expand() {
    this.collapsed = false;
  }

  collapse() {
    this.collapsed = true;
  }

  setActive(route: string) {
    this.activeRoute = route;
  }

  isActive(route: string): boolean {
    if (route === '/') return this.activeRoute === '/';
    return this.activeRoute.startsWith(route);
  }

  updateBadge(itemId: string, count: number) {
    for (const group of this.groups) {
      const item = group.items.find(i => i.id === itemId);
      if (item) {
        item.badge = count;
        break;
      }
    }
  }
}

export const navigationStore = new NavigationStore();
