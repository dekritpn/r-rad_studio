<!-- src/lib/components/preferences/PreferencesModal.svelte -->
<!-- Application Preferences Modal for Theme, Typography, and File Paths -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import { preferencesStore, type UserPreferences, DEFAULT_PREFERENCES } from '../../stores/preferencesStore.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let activeTab = $state<'appearance' | 'typography' | 'paths'>('appearance');

  // Local draft state for editing
  let formState = $state<UserPreferences>({ ...preferencesStore.current });

  // Whenever the modal opens, copy current preferences into formState
  $effect(() => {
    if (appState.isPreferencesOpen) {
      formState = { ...preferencesStore.current };
    }
  });

  const fontOptions = [
    { label: 'Inter (Default Modern Sans)', value: 'Inter, system-ui, sans-serif' },
    { label: 'Roboto', value: 'Roboto, sans-serif' },
    { label: 'Segoe UI / Tahoma', value: '"Segoe UI", Tahoma, sans-serif' },
    { label: 'Fira Sans', value: '"Fira Sans", sans-serif' },
    { label: 'JetBrains Mono (Monospace)', value: '"JetBrains Mono", monospace' },
    { label: 'System Default Sans', value: 'system-ui, -apple-system, sans-serif' },
  ];

  async function browseFolder(field: 'modules_dir' | 'output_dir', title: string) {
    try {
      const selected: string | null = await invoke('pick_folder', { title });
      if (selected) {
        formState[field] = selected;
      }
    } catch (err) {
      console.error('Failed to pick folder:', err);
    }
  }

  async function handleSave() {
    await preferencesStore.save(formState);
    appState.showToast('Preferences saved and applied successfully');
    appState.isPreferencesOpen = false;
  }

  async function handleResetDefaults() {
    formState = { ...DEFAULT_PREFERENCES };
    await preferencesStore.resetToDefaults();
    appState.showToast('Reset preferences to default values');
  }

  function handleClose() {
    appState.isPreferencesOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if appState.isPreferencesOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div
      class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col w-full max-w-2xl max-h-[85vh] overflow-hidden text-slate-100"
      role="dialog"
      aria-modal="true"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-slate-800 bg-slate-950 flex items-center justify-between select-none">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-lg bg-indigo-600/30 border border-indigo-500/50 flex items-center justify-center text-indigo-400 font-bold text-sm">
            ⚙️
          </div>
          <div>
            <h2 class="text-sm font-semibold text-white tracking-wide">Application Preferences</h2>
            <p class="text-[11px] text-slate-400">Configure Appearance, Fonts, and Directory Paths</p>
          </div>
        </div>

        <!-- Navigation Tabs -->
        <div class="flex items-center bg-slate-900 border border-slate-800 rounded-lg p-0.5 text-xs">
          <button
            type="button"
            class="px-3 py-1.5 rounded-md font-medium transition-all {activeTab === 'appearance' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => activeTab = 'appearance'}
          >
            🎨 Appearance
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-md font-medium transition-all {activeTab === 'typography' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => activeTab = 'typography'}
          >
            🔤 Typography
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-md font-medium transition-all {activeTab === 'paths' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => activeTab = 'paths'}
          >
            📁 Directories
          </button>
        </div>

        <!-- Close Button -->
        <button
          type="button"
          class="text-slate-400 hover:text-white p-1 rounded-md hover:bg-slate-800 transition-colors"
          onclick={handleClose}
          title="Close (Esc)"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6 text-xs text-slate-300">
        {#if activeTab === 'appearance'}
          <!-- THEME SELECTION -->
          <div class="space-y-4">
            <div>
              <h3 class="text-sm font-semibold text-white mb-1">Color Theme</h3>
              <p class="text-slate-400 text-[11px]">Select your preferred visual style across the workbench.</p>
            </div>

            <div class="grid grid-cols-3 gap-3">
              <!-- Dark Theme Option -->
              <label class="cursor-pointer">
                <input
                  type="radio"
                  name="theme"
                  value="dark"
                  bind:group={formState.theme}
                  class="sr-only"
                />
                <div class="p-3.5 rounded-xl border-2 transition-all flex flex-col items-center gap-2 text-center {formState.theme === 'dark' ? 'border-indigo-500 bg-indigo-950/30 text-white shadow-md' : 'border-slate-800 bg-slate-950/60 hover:border-slate-700'}">
                  <div class="w-full h-12 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center text-slate-300 font-mono text-[11px]">
                    Dark Slate
                  </div>
                  <span class="font-semibold text-xs mt-1">Dark Mode</span>
                  <span class="text-[10px] text-slate-400">Low-glare dark palette</span>
                </div>
              </label>

              <!-- Light Theme Option -->
              <label class="cursor-pointer">
                <input
                  type="radio"
                  name="theme"
                  value="light"
                  bind:group={formState.theme}
                  class="sr-only"
                />
                <div class="p-3.5 rounded-xl border-2 transition-all flex flex-col items-center gap-2 text-center {formState.theme === 'light' ? 'border-indigo-500 bg-indigo-950/30 text-white shadow-md' : 'border-slate-800 bg-slate-950/60 hover:border-slate-700'}">
                  <div class="w-full h-12 rounded-lg bg-slate-100 border border-slate-300 flex items-center justify-center text-slate-800 font-mono text-[11px]">
                    Clean White
                  </div>
                  <span class="font-semibold text-xs mt-1">Light Mode</span>
                  <span class="text-[10px] text-slate-400">High contrast bright palette</span>
                </div>
              </label>

              <!-- System Default Option -->
              <label class="cursor-pointer">
                <input
                  type="radio"
                  name="theme"
                  value="system"
                  bind:group={formState.theme}
                  class="sr-only"
                />
                <div class="p-3.5 rounded-xl border-2 transition-all flex flex-col items-center gap-2 text-center {formState.theme === 'system' ? 'border-indigo-500 bg-indigo-950/30 text-white shadow-md' : 'border-slate-800 bg-slate-950/60 hover:border-slate-700'}">
                  <div class="w-full h-12 rounded-lg bg-gradient-to-r from-slate-900 to-slate-100 border border-slate-700 flex items-center justify-center text-indigo-300 font-mono text-[11px]">
                    Auto OS
                  </div>
                  <span class="font-semibold text-xs mt-1">System Default</span>
                  <span class="text-[10px] text-slate-400">Matches OS appearance</span>
                </div>
              </label>
            </div>
          </div>

        {:else if activeTab === 'typography'}
          <!-- TYPOGRAPHY SETTINGS -->
          <div class="space-y-5">
            <div>
              <h3 class="text-sm font-semibold text-white mb-1">Typography & Font Sizes</h3>
              <p class="text-slate-400 text-[11px]">Customize font family and font size individually for each application window.</p>
            </div>

            <!-- 1. Menu Font -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-200">Bilah Menu (Menu Bar)</span>
                <span class="font-mono text-indigo-400 font-bold text-xs">{formState.font_size_menu}px</span>
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Family</label>
                  <select
                    bind:value={formState.font_family_ui}
                    class="w-full bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                  >
                    {#each fontOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Size (9px – 18px)</label>
                  <input
                    type="range"
                    min="9"
                    max="18"
                    step="1"
                    bind:value={formState.font_size_menu}
                    class="w-full mt-2 accent-indigo-500"
                  />
                </div>
              </div>
            </div>

            <!-- 2. Data Window Font -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-200">Jendela Data (Data View & Variable View)</span>
                <span class="font-mono text-indigo-400 font-bold text-xs">{formState.font_size_data}px</span>
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Family</label>
                  <select
                    bind:value={formState.font_family_data}
                    class="w-full bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                  >
                    {#each fontOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Size (9px – 20px)</label>
                  <input
                    type="range"
                    min="9"
                    max="20"
                    step="1"
                    bind:value={formState.font_size_data}
                    class="w-full mt-2 accent-indigo-500"
                  />
                </div>
              </div>
            </div>

            <!-- 3. Output Window Font -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-200">Jendela Output (Tables & Statistical Log)</span>
                <span class="font-mono text-indigo-400 font-bold text-xs">{formState.font_size_output}px</span>
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Family</label>
                  <select
                    bind:value={formState.font_family_output}
                    class="w-full bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                  >
                    {#each fontOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Size (10px – 22px)</label>
                  <input
                    type="range"
                    min="10"
                    max="22"
                    step="1"
                    bind:value={formState.font_size_output}
                    class="w-full mt-2 accent-indigo-500"
                  />
                </div>
              </div>
            </div>

            <!-- 4. Help & Dialogs Font -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-3">
              <div class="flex items-center justify-between">
                <span class="font-semibold text-slate-200">Jendela Help & Dialog Modul</span>
                <span class="font-mono text-indigo-400 font-bold text-xs">{formState.font_size_help}px</span>
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Family</label>
                  <select
                    bind:value={formState.font_family_help}
                    class="w-full bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                  >
                    {#each fontOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class="text-[10px] uppercase font-bold text-slate-400 block mb-1">Font Size (9px – 18px)</label>
                  <input
                    type="range"
                    min="9"
                    max="18"
                    step="1"
                    bind:value={formState.font_size_help}
                    class="w-full mt-2 accent-indigo-500"
                  />
                </div>
              </div>
            </div>
          </div>

        {:else if activeTab === 'paths'}
          <!-- DIRECTORIES & PATHS -->
          <div class="space-y-5">
            <div>
              <h3 class="text-sm font-semibold text-white mb-1">Folder & Directory Paths</h3>
              <p class="text-slate-400 text-[11px]">Configure custom directories for modules and exported outputs.</p>
            </div>

            <!-- Modules Directory -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-2">
              <label class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
                <span>📦 Folder Modules</span>
              </label>
              <div class="flex items-center gap-2">
                <input
                  type="text"
                  bind:value={formState.modules_dir}
                  class="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-100 focus:outline-none focus:border-indigo-500"
                  placeholder="/path/to/modules"
                />
                <button
                  type="button"
                  class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg font-medium text-xs transition-colors shrink-0"
                  onclick={() => browseFolder('modules_dir', 'Select Modules Directory')}
                >
                  📁 Browse...
                </button>
              </div>
              <p class="text-[10px] text-slate-500 leading-tight">
                R-RAD Studio automatically builds the <strong>Analyze</strong> menu based on the folder hierarchy in this directory.
              </p>
            </div>

            <!-- Output Directory -->
            <div class="p-3.5 bg-slate-950 rounded-xl border border-slate-800 space-y-2">
              <label class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
                <span>💾 Folder Output</span>
              </label>
              <div class="flex items-center gap-2">
                <input
                  type="text"
                  bind:value={formState.output_dir}
                  class="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-100 focus:outline-none focus:border-indigo-500"
                  placeholder="/path/to/output"
                />
                <button
                  type="button"
                  class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg font-medium text-xs transition-colors shrink-0"
                  onclick={() => browseFolder('output_dir', 'Select Default Output Directory')}
                >
                  📁 Browse...
                </button>
              </div>
              <p class="text-[10px] text-slate-500 leading-tight">
                Default location for exported HTML reports, publication-ready tables, and vector graphics.
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-slate-800 bg-slate-950 flex items-center justify-between text-xs">
        <button
          type="button"
          class="px-3 py-1.5 bg-slate-900 hover:bg-rose-950/60 text-slate-400 hover:text-rose-300 border border-slate-800 hover:border-rose-800 rounded-lg font-medium transition-colors"
          onclick={handleResetDefaults}
        >
          ↺ Reset to Defaults
        </button>

        <div class="flex items-center gap-2">
          <button
            type="button"
            class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg font-medium transition-colors"
            onclick={handleClose}
          >
            Cancel
          </button>
          <button
            type="button"
            class="px-4 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-bold shadow-md transition-colors"
            onclick={handleSave}
          >
            💾 Save & Apply
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
