<!-- src/App.svelte -->
<!-- R-RAD Studio Main Application Workbench Shell -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { appState } from './lib/stores/appState.svelte';
  import MenuBar from './lib/components/MenuBar.svelte';
  import DataGrid from './lib/components/grid/DataGrid.svelte';
  import OutputViewer from './lib/components/output/OutputViewer.svelte';
  import FormDesigner from './lib/components/designer/FormDesigner.svelte';
  import FormModal from './lib/components/runtime/FormModal.svelte';
  import ComputeVariableModal from './lib/components/wrangling/ComputeVariableModal.svelte';
  import HelpModal from './lib/components/help/HelpModal.svelte';
  import PreferencesModal from './lib/components/preferences/PreferencesModal.svelte';
  import { preferencesStore } from './lib/stores/preferencesStore.svelte';
  import { invoke } from '@tauri-apps/api/core';

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if (e.key === 'F1') {
      e.preventDefault();
      appState.openHelp('user');
    } else if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      appState.openPreferences();
    }
  }

  onMount(async () => {
    // 0. Initialize persistent user preferences (Theme, Typography, Paths)
    await preferencesStore.init();

    try {
      // 1. Handshake with R Daemon
      appState.daemonStatus = 'running';
      const pingRes: any = await invoke('ping_r');
      appState.rVersion = pingRes.version || 'R Session Active';
      appState.daemonStatus = 'idle';

      // 2. Fetch Form Registry Menu
      const menu: any = await invoke('get_forms_menu');
      appState.menuTree = menu;

      // 3. Load default dataset (mtcars)
      const meta: any = await invoke('load_dataset', { filePath: 'mtcars' });
      appState.setDataset(meta);

      const slice: any = await invoke('get_data_page', { offset: 0, limit: appState.dataLimit });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
    } catch (err) {
      console.error('Initialization error:', err);
      appState.daemonStatus = 'error';
    }
  });
</script>

<svelte:window onkeydown={handleGlobalKeyDown} />

<div class="flex flex-col h-screen w-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
  <!-- Top Menu Bar -->
  <MenuBar />

  <!-- Main View Workspace -->
  <main class="flex-1 overflow-hidden relative">
    {#if appState.viewMode === 'data' || appState.viewMode === 'variable'}
      <DataGrid />
    {:else if appState.viewMode === 'output'}
      <OutputViewer />
    {:else if appState.viewMode === 'designer'}
      <FormDesigner />
    {/if}
  </main>

  <!-- Runtime Dialog Modal -->
  <FormModal />

  <!-- Compute Variable Modal -->
  <ComputeVariableModal />

  <!-- In-App Help & About Modal -->
  <HelpModal />

  <!-- Application Preferences Modal -->
  <PreferencesModal />

  <!-- Global Toast Notification -->
  {#if appState.toastMessage}
    <div class="fixed bottom-6 right-6 z-[100] bg-indigo-600 text-white px-4 py-2.5 rounded-xl shadow-2xl text-xs font-semibold flex items-center gap-2 border border-indigo-400/40">
      <span>🔔</span>
      <span>{appState.toastMessage}</span>
    </div>
  {/if}
</div>
