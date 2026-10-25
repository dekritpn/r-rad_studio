// src/lib/stores/preferencesStore.svelte.ts
// Persistent user preferences for theme, typography, modules directory, and output directory.

import { invoke } from '@tauri-apps/api/core';

export interface UserPreferences {
  theme: 'dark' | 'light' | 'system';
  font_family_ui: string;
  font_family_data: string;
  font_family_output: string;
  font_family_help: string;
  font_size_menu: number;
  font_size_data: number;
  font_size_output: number;
  font_size_help: number;
  modules_dir: string;
  output_dir: string;
}

export const DEFAULT_PREFERENCES: UserPreferences = {
  theme: 'dark',
  font_family_ui: 'Inter, system-ui, sans-serif',
  font_family_data: 'Inter, system-ui, sans-serif',
  font_family_output: 'Inter, system-ui, sans-serif',
  font_family_help: 'Inter, system-ui, sans-serif',
  font_size_menu: 12,
  font_size_data: 12,
  font_size_output: 13,
  font_size_help: 12,
  modules_dir: '/home/dpng/Projects/r_rad_studio/modules',
  output_dir: '/home/dpng/Documents/R-RAD_Output',
};

class PreferencesManager {
  current = $state<UserPreferences>({ ...DEFAULT_PREFERENCES });
  isInitialized = $state<boolean>(false);

  async init() {
    try {
      const saved: any = await invoke('load_user_preferences');
      if (saved && typeof saved === 'object') {
        this.current = {
          ...DEFAULT_PREFERENCES,
          ...saved,
        };
      }
    } catch (err) {
      console.warn('Failed to load user preferences via IPC, falling back to localStorage/defaults:', err);
      const local = localStorage.getItem('rrad_preferences');
      if (local) {
        try {
          this.current = { ...DEFAULT_PREFERENCES, ...JSON.parse(local) };
        } catch {}
      }
    } finally {
      this.isInitialized = true;
      this.applyToDOM();
    }
  }

  applyToDOM() {
    if (typeof document === 'undefined') return;

    const root = document.documentElement;

    // Apply CSS Variables for Fonts and Font Sizes
    root.style.setProperty('--font-menu', this.current.font_family_ui);
    root.style.setProperty('--size-menu', `${this.current.font_size_menu}px`);

    root.style.setProperty('--font-data', this.current.font_family_data);
    root.style.setProperty('--size-data', `${this.current.font_size_data}px`);

    root.style.setProperty('--font-output', this.current.font_family_output);
    root.style.setProperty('--size-output', `${this.current.font_size_output}px`);

    root.style.setProperty('--font-help', this.current.font_family_help);
    root.style.setProperty('--size-help', `${this.current.font_size_help}px`);

    // Apply Theme (Dark / Light / System)
    let isLight = false;
    if (this.current.theme === 'light') {
      isLight = true;
    } else if (this.current.theme === 'system') {
      isLight = window.matchMedia && !window.matchMedia('(prefers-color-scheme: dark)').matches;
    }

    if (isLight) {
      root.classList.add('theme-light');
      root.classList.remove('theme-dark');
      root.style.colorScheme = 'light';
    } else {
      root.classList.add('theme-dark');
      root.classList.remove('theme-light');
      root.style.colorScheme = 'dark';
    }
  }

  async save(newPrefs: UserPreferences) {
    this.current = { ...newPrefs };
    this.applyToDOM();

    // Cache in localStorage
    try {
      localStorage.setItem('rrad_preferences', JSON.stringify(newPrefs));
    } catch {}

    // Persist via backend
    try {
      await invoke('save_user_preferences', { preferences: newPrefs });
    } catch (err) {
      console.error('Failed to persist preferences via IPC:', err);
    }
  }

  async resetToDefaults() {
    await this.save(DEFAULT_PREFERENCES);
  }
}

export const preferencesStore = new PreferencesManager();
