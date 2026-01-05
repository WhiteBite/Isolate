<script lang="ts">
  import { getComponent, executeCommand, type PluginComponent } from './plugin-host';

  interface Props {
    pluginId: string;
    componentName: string;
  }

  let { pluginId, componentName }: Props = $props();

  let container: HTMLDivElement;
  let component: PluginComponent | undefined;
  let data: any = $state({});
  let unsubscribers: (() => void)[] = [];

  // Setup and cleanup with $effect
  $effect(() => {
    component = getComponent(pluginId, componentName);
    if (!component) {
      console.error(`Component not found: ${pluginId}.${componentName}`);
      return;
    }

    // Initialize data
    if (component.data) {
      data = component.data();
    }

    // Inject styles
    if (component.styles) {
      const style = document.createElement('style');
      style.textContent = component.styles;
      style.setAttribute('data-plugin', pluginId);
      document.head.appendChild(style);
    }

    // Listen for plugin events
    const eventHandler = (e: CustomEvent) => {
      if (component?.data) {
        data = component.data();
        renderTemplate();
      }
    };

    // Subscribe to all plugin events
    window.addEventListener(`plugin:${pluginId}:results`, eventHandler as EventListener);
    window.addEventListener(`plugin:${pluginId}:checking`, eventHandler as EventListener);
    
    unsubscribers.push(
      () => window.removeEventListener(`plugin:${pluginId}:results`, eventHandler as EventListener),
      () => window.removeEventListener(`plugin:${pluginId}:checking`, eventHandler as EventListener)
    );

    renderTemplate();
    
    return () => {
      unsubscribers.forEach(fn => fn());
      // Remove injected styles
      document.querySelectorAll(`style[data-plugin="${pluginId}"]`).forEach(el => el.remove());
    };
  });

  function renderTemplate() {
    if (!component || !container) return;

    let html = component.template;

    // Simple template engine
    // Replace {variable} with data values
    html = html.replace(/\{([^}]+)\}/g, (match, expr) => {
      try {
        // Handle method calls
        if (expr.includes('(')) {
          const methodName = expr.split('(')[0].trim();
          if (component?.methods?.[methodName]) {
            const args = expr.match(/\(([^)]*)\)/)?.[1];
            if (args) {
              const argValue = evaluateExpression(args, data);
              return String(component.methods[methodName](argValue));
            }
            return String(component.methods[methodName]());
          }
        }
        return String(evaluateExpression(expr, data));
      } catch {
        return match;
      }
    });

    // Handle if="{condition}"
    html = html.replace(/<([^>]+)\s+if="\{([^}]+)\}"([^>]*)>([\s\S]*?)<\/\1>/g, 
      (match, tag, condition, attrs, content) => {
        const result = evaluateExpression(condition, data);
        return result ? `<${tag}${attrs}>${content}</${tag}>` : '';
      }
    );

    // Handle each loops
    html = html.replace(/<each\s+items="\{([^}]+)\}"\s+as="([^"]+)">([\s\S]*?)<\/each>/g,
      (match, itemsExpr, itemName, template) => {
        const items = evaluateExpression(itemsExpr, data);
        if (!Array.isArray(items)) return '';
        return items.map((item, index) => {
          let itemHtml = template;
          // Replace {item.prop} and {item}
          itemHtml = itemHtml.replace(new RegExp(`\\{${itemName}\\.([^}]+)\\}`, 'g'), 
            (m, prop) => String(item[prop] ?? ''));
          itemHtml = itemHtml.replace(new RegExp(`\\{${itemName}\\}`, 'g'), String(item));
          return itemHtml;
        }).join('');
      }
    );

    container.innerHTML = html;

    // Bind onclick handlers
    container.querySelectorAll('[onclick]').forEach(el => {
      const methodName = el.getAttribute('onclick');
      if (methodName && component?.methods?.[methodName]) {
        el.removeAttribute('onclick');
        el.addEventListener('click', () => {
          component!.methods![methodName]();
          // Re-render after action
          setTimeout(() => {
            if (component?.data) {
              data = component.data();
              renderTemplate();
            }
          }, 100);
        });
      }
    });
  }

  function evaluateExpression(expr: string, context: any): any {
    // Handle ternary
    if (expr.includes('?')) {
      const [condition, rest] = expr.split('?');
      const [truthy, falsy] = rest.split(':');
      return evaluateExpression(condition.trim(), context) 
        ? evaluateExpression(truthy.trim(), context)
        : evaluateExpression(falsy.trim(), context);
    }

    // Handle property access
    const parts = expr.trim().split('.');
    let value: any = context;
    
    for (const part of parts) {
      if (value === null || value === undefined) return undefined;
      // Handle array access like results[0]
      const arrayMatch = part.match(/(\w+)\[(\d+)\]/);
      if (arrayMatch) {
        value = value[arrayMatch[1]]?.[parseInt(arrayMatch[2])];
      } else {
        value = value[part];
      }
    }

    // Handle method calls on value
    if (typeof value === 'function') {
      return value();
    }

    return value;
  }
</script>

<div class="plugin-view" bind:this={container}>
  {#if !component}
    <div class="plugin-error">
      <p>Компонент не найден: {pluginId}.{componentName}</p>
    </div>
  {/if}
</div>

<style>
  .plugin-view {
    width: 100%;
    height: 100%;
  }
  .plugin-error {
    padding: 1rem;
    color: #ff3333;
    text-align: center;
  }
</style>
