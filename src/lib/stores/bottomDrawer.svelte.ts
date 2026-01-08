/**
 * Bottom Drawer Store - Svelte 5 runes
 * Manages the bottom drawer (logs panel) state
 */

class BottomDrawerStore {
  isOpen = $state(false);
  height = $state(35); // процент от высоты экрана
  minHeight = 20;
  maxHeight = 70;
  
  // Для drag resize
  isDragging = $state(false);
  
  open() {
    this.isOpen = true;
  }
  
  close() {
    this.isOpen = false;
  }
  
  toggle() {
    this.isOpen = !this.isOpen;
  }
  
  setHeight(h: number) {
    this.height = Math.max(this.minHeight, Math.min(this.maxHeight, h));
  }
  
  startDrag() {
    this.isDragging = true;
  }
  
  stopDrag() {
    this.isDragging = false;
  }
}

export const bottomDrawerStore = new BottomDrawerStore();
