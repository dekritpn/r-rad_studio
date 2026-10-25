<!-- src/lib/components/MenuBar.svelte -->
<!-- SPSS-Style Main Menu Bar with Dynamic Forms Catalog Integration -->
<script lang="ts">
  import { appState } from '../stores/appState.svelte';
  import type { MenuNode } from '../schemas/rform';
  import { invoke } from '@tauri-apps/api/core';

  let activeMenu = $state<string | null>(null);

  // Close menus on outside click
  function closeMenus() {
    activeMenu = null;
  }

  async function openFormDialog(formId: string) {
    try {
      const schema: any = await invoke('get_form_by_id', { id: formId });
      appState.openDialog(schema);
    } catch (err) {
      console.error(`Failed to load form ${formId}:`, err);
    } finally {
      closeMenus();
    }
  }

  async function loadSampleDataset(name: string) {
    try {
      appState.daemonStatus = 'running';
      const meta: any = await invoke('load_dataset', { filePath: name });
      appState.setDataset(meta);
      // Load first page of data
      const slice: any = await invoke('get_data_page', { offset: 0, limit: appState.dataLimit });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
      appState.daemonStatus = 'idle';
      appState.setView('data');
    } catch (err) {
      console.error(`Failed to load dataset ${name}:`, err);
      appState.daemonStatus = 'error';
    } finally {
      closeMenus();
    }
  }

  async function resetRSession() {
    try {
      appState.daemonStatus = 'running';
      await invoke('cancel_analysis');
      appState.daemonStatus = 'idle';
    } catch (err) {
      console.error('Failed to reset R session:', err);
    } finally {
      closeMenus();
    }
  }
  async function openImportDatasetDialog() {
    try {
      const selectedPath: string | null = await invoke('pick_file', {
        title: 'Open / Import Dataset',
        filterName: 'Dataset Files (*.sav, *.csv, *.xlsx, *.xls, *.rds)',
        extensions: ['sav', 'csv', 'xlsx', 'xls', 'rds'],
      });
      if (selectedPath) {
        await loadSampleDataset(selectedPath);
        appState.showToast(`Imported dataset: ${selectedPath.split('/').pop()}`);
      }
    } catch (err) {
      console.error('Failed to import dataset:', err);
      appState.showToast(`Import error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeMenus();
    }
  }

  async function exportDatasetAs(format: 'csv' | 'sav' | 'xlsx') {
    if (!appState.activeDataset) {
      appState.showToast('No active dataset to export');
      closeMenus();
      return;
    }
    try {
      const defaultName = `${appState.activeDataset.dataset_name || 'dataset'}.${format}`;
      const savePath: string | null = await invoke('save_file', {
        title: `Export Dataset as ${format.toUpperCase()}`,
        defaultName,
        filterName: `${format.toUpperCase()} Files (*.${format})`,
        extensions: [format],
      });
      if (savePath) {
        appState.daemonStatus = 'running';
        await invoke('export_dataset', {
          filePath: savePath,
          format,
        });
        appState.daemonStatus = 'idle';
        appState.showToast(`Exported dataset successfully to ${savePath.split('/').pop()}`);
      }
    } catch (err) {
      console.error(`Failed to export dataset as ${format}:`, err);
      appState.daemonStatus = 'error';
      appState.showToast(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeMenus();
    }
  }

  async function saveProjectAsRrad() {
    try {
      const defaultName = `${appState.activeDataset?.dataset_name || 'project'}.rrad`;
      const savePath: string | null = await invoke('save_file', {
        title: 'Save R-RAD Project',
        defaultName,
        filterName: 'R-RAD Project Files (*.rrad)',
        extensions: ['rrad', 'json'],
      });
      if (savePath) {
        let fullRows = appState.dataRows;
        if (appState.activeDataset && appState.totalRows > appState.dataRows.length) {
          const slice: any = await invoke('get_data_page', { offset: 0, limit: appState.totalRows });
          fullRows = slice.rows;
        }

        const projectData = {
          $schema: 'https://r-rad-studio.local/schemas/v1/project.json',
          version: '1.0.0',
          project_meta: {
            title: appState.activeDataset?.dataset_name || 'Untitled Project',
            saved_at: new Date().toISOString(),
            r_version: appState.rVersion || 'R Active Session',
          },
          dataset: appState.activeDataset ? {
            dataset_name: appState.activeDataset.dataset_name,
            row_count: appState.totalRows,
            column_count: appState.activeDataset.column_count,
            variables: appState.activeDataset.variables,
            rows: fullRows,
          } : null,
          syntax_history: appState.syntaxCode,
          output_logs: appState.outputLogs,
        };

        await invoke('save_project', {
          filePath: savePath,
          projectData,
        });
        appState.showToast(`Project saved to ${savePath.split('/').pop()}`);
      }
    } catch (err) {
      console.error('Failed to save project:', err);
      appState.showToast(`Save project error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeMenus();
    }
  }

  async function openProjectRrad() {
    try {
      const selectedPath: string | null = await invoke('pick_file', {
        title: 'Open R-RAD Project',
        filterName: 'R-RAD Project Files (*.rrad, *.json)',
        extensions: ['rrad', 'json'],
      });
      if (selectedPath) {
        appState.daemonStatus = 'running';
        const projectData: any = await invoke('open_project', {
          filePath: selectedPath,
        });

        if (projectData.dataset) {
          appState.setDataset({
            dataset_name: projectData.dataset.dataset_name,
            row_count: projectData.dataset.row_count,
            column_count: projectData.dataset.column_count,
            variables: projectData.dataset.variables,
          });
          const slice: any = await invoke('get_data_page', { offset: 0, limit: appState.dataLimit });
          appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
        }

        if (Array.isArray(projectData.output_logs)) {
          appState.outputLogs = projectData.output_logs;
          if (appState.outputLogs.length > 0) {
            appState.selectedLogId = appState.outputLogs[appState.outputLogs.length - 1].id;
          }
        }

        if (typeof projectData.syntax_history === 'string') {
          appState.syntaxCode = projectData.syntax_history;
        }

        appState.daemonStatus = 'idle';
        appState.showToast(`Loaded project: ${selectedPath.split('/').pop()}`);
        appState.setView('data');
      }
    } catch (err) {
      console.error('Failed to open project:', err);
      appState.daemonStatus = 'error';
      appState.showToast(`Open project error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeMenus();
    }
  }

  async function insertBlankVariable() {
    try {
      const nextNum = (appState.activeDataset?.column_count ?? 0) + 1;
      const newVarName = `VAR${String(nextNum).padStart(5, '0')}`;
      const meta: any = await invoke('add_variable', {
        name: newVarName,
        measure: 'scale',
        label: newVarName,
      });
      appState.setDataset(meta);
      const slice: any = await invoke('get_data_page', { offset: appState.dataOffset, limit: appState.dataLimit });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
      appState.showToast(`Added new blank column '${newVarName}'`);
      appState.setView('data');
    } catch (err) {
      console.error('Failed to add variable:', err);
      appState.showToast(`Error adding variable: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeMenus();
    }
  }
</script>

<svelte:window onclick={closeMenus} />

<div class="h-9 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-2 text-xs select-none z-50">
  <!-- Left Side: Classic Menus -->
  <div class="flex items-center gap-0.5" onclick={(e) => e.stopPropagation()}>
    <!-- FILE MENU -->
    <div class="relative">
      <button
        class="px-2.5 py-1 rounded hover:bg-slate-800 transition-colors {activeMenu === 'file' ? 'bg-slate-800 text-white' : 'text-slate-300'}"
        onclick={() => activeMenu = activeMenu === 'file' ? null : 'file'}
      >
        File
      </button>
      {#if activeMenu === 'file'}
        <div class="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50 divide-y divide-slate-800">
          <!-- Project Operations -->
          <div class="py-1">
            <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400 tracking-wider">Project Session</div>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
              onclick={openProjectRrad}
            >
              <span>📂 Open Project (.rrad)...</span>
            </button>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
              onclick={saveProjectAsRrad}
            >
              <span>💾 Save Project (.rrad)...</span>
            </button>
          </div>

          <!-- Dataset Import / Export -->
          <div class="py-1">
            <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400 tracking-wider">Dataset I/O</div>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={openImportDatasetDialog}
            >
              <span>📥 Open / Import Dataset...</span>
              <span class="text-[9px] text-slate-400 font-mono">SAV/CSV/XLSX</span>
            </button>
            <div class="px-3 pt-2 pb-1 text-[10px] uppercase font-semibold text-slate-500">Export Dataset As:</div>
            <button
              class="w-full text-left px-4 py-1 text-slate-200 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-[11px] transition-colors"
              onclick={() => exportDatasetAs('csv')}
            >
              <span>Comma Separated (.csv)</span>
              <span class="text-[10px] text-emerald-400 font-mono">CSV</span>
            </button>
            <button
              class="w-full text-left px-4 py-1 text-slate-200 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-[11px] transition-colors"
              onclick={() => exportDatasetAs('sav')}
            >
              <span>SPSS Dataset (.sav)</span>
              <span class="text-[10px] text-amber-400 font-mono">SAV</span>
            </button>
            <button
              class="w-full text-left px-4 py-1 text-slate-200 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-[11px] transition-colors"
              onclick={() => exportDatasetAs('xlsx')}
            >
              <span>Microsoft Excel (.xlsx)</span>
              <span class="text-[10px] text-sky-400 font-mono">XLSX</span>
            </button>
          </div>

          <!-- Sample Datasets -->
          <div class="py-1">
            <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400 tracking-wider">Sample Datasets</div>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={() => loadSampleDataset('mtcars')}
            >
              <span>Motor Trend Cars (mtcars)</span>
              <span class="text-[10px] text-slate-400">32 rows</span>
            </button>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={() => loadSampleDataset('iris')}
            >
              <span>Fisher's Iris (iris)</span>
              <span class="text-[10px] text-slate-400">150 rows</span>
            </button>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={() => loadSampleDataset('/home/dpng/Projects/r_rad_studio/src-tauri/tests/fixtures/test_50k.sav')}
            >
              <span>50,000 Rows (.sav)</span>
              <span class="text-[10px] text-amber-400 font-semibold">50k rows</span>
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- DATA MENU -->
    <div class="relative">
      <button
        class="px-2.5 py-1 rounded hover:bg-slate-800 transition-colors {activeMenu === 'data' ? 'bg-slate-800 text-white' : 'text-slate-300'}"
        onclick={() => activeMenu = activeMenu === 'data' ? null : 'data'}
      >
        Data
      </button>
      {#if activeMenu === 'data'}
        <div class="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1.5 z-50 divide-y divide-slate-800 text-xs">
          <div class="py-1">
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
              onclick={insertBlankVariable}
            >
              <span>➕ Insert Variable (Blank Column)...</span>
            </button>
          </div>
          <div class="py-1">
            <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400 tracking-wider">Transformations</div>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
              onclick={() => { appState.isComputeDialogOpen = true; closeMenus(); }}
            >
              <span>🧮 Compute Variable...</span>
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- VIEW MENU -->
    <div class="relative">
      <button
        class="px-2.5 py-1 rounded hover:bg-slate-800 transition-colors {activeMenu === 'view' ? 'bg-slate-800 text-white' : 'text-slate-300'}"
        onclick={() => activeMenu = activeMenu === 'view' ? null : 'view'}
      >
        View
      </button>
      {#if activeMenu === 'view'}
        <div class="absolute top-full left-0 mt-1 w-52 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50">
          <button
            class="w-full text-left px-3 py-2 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors {appState.viewMode === 'data' ? 'text-indigo-400 font-semibold' : ''}"
            onclick={() => { appState.setView('data'); closeMenus(); }}
          >
            <span>Dual Data Grid</span>
          </button>
          <button
            class="w-full text-left px-3 py-2 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors {appState.viewMode === 'output' ? 'text-indigo-400 font-semibold' : ''}"
            onclick={() => { appState.setView('output'); closeMenus(); }}
          >
            <span>Output Viewer</span>
          </button>
          <button
            class="w-full text-left px-3 py-2 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors {appState.viewMode === 'designer' ? 'text-indigo-400 font-semibold' : ''}"
            onclick={() => { appState.setView('designer'); closeMenus(); }}
          >
            <span>Form Designer (RAD Mode)</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- ANALYZE MENU (Hierarchical from Registry) -->
    <div class="relative">
      <button
        class="px-2.5 py-1 rounded hover:bg-slate-800 transition-colors {activeMenu === 'analyze' ? 'bg-slate-800 text-white' : 'text-slate-300'}"
        onclick={() => activeMenu = activeMenu === 'analyze' ? null : 'analyze'}
      >
        Analyze
      </button>
      {#if activeMenu === 'analyze'}
        <div class="absolute top-full left-0 mt-1 w-72 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1.5 z-50 divide-y divide-slate-800">
          <!-- Render dynamic categories following modules/ folder hierarchy -->
          {#each appState.menuTree.find(n => n.label === 'Analyze')?.children || [] as cat}
            <div class="py-1">
              {#if cat.children && cat.children.length > 0}
                <div class="px-3 py-1 text-[10px] uppercase font-bold text-indigo-400 tracking-wider">
                  {cat.label}
                </div>
                {#each cat.children as item}
                  {#if item.children && item.children.length > 0}
                    <!-- Subfolder / Submenu Flyout -->
                    <div class="relative group/sub">
                      <button
                        type="button"
                        class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-xs transition-colors"
                      >
                        <span class="truncate">{item.label}</span>
                        <span class="text-[9px] text-slate-400 group-hover/sub:text-white ml-2 font-mono">▶</span>
                      </button>
                      <div class="hidden group-hover/sub:block absolute left-full top-0 ml-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 z-50">
                        <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400 tracking-wider border-b border-slate-800 mb-1">
                          {item.label}
                        </div>
                        {#each item.children as subItem}
                          <button
                            type="button"
                            class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-xs transition-colors"
                            onclick={() => subItem.form_id && openFormDialog(subItem.form_id)}
                          >
                            <span class="truncate">{subItem.label}</span>
                            {#if subItem.is_custom}
                              <span class="text-[9px] bg-indigo-950 text-indigo-300 px-1 py-0.5 rounded border border-indigo-700">Custom</span>
                            {/if}
                          </button>
                        {/each}
                      </div>
                    </div>
                  {:else}
                    <!-- Direct Form Item in Folder -->
                    <button
                      type="button"
                      class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between text-xs transition-colors"
                      onclick={() => item.form_id && openFormDialog(item.form_id)}
                    >
                      <span class="truncate">{item.label}</span>
                      {#if item.is_custom}
                        <span class="text-[9px] bg-indigo-950 text-indigo-300 px-1 py-0.5 rounded border border-indigo-700">Custom</span>
                      {/if}
                    </button>
                  {/if}
                {/each}
              {:else if cat.form_id}
                <!-- Root Module Item in modules/ -->
                <button
                  type="button"
                  class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white text-xs transition-colors"
                  onclick={() => cat.form_id && openFormDialog(cat.form_id)}
                >
                  {cat.label}
                </button>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>


    <!-- HELP MENU -->
    <div class="relative">
      <button
        class="px-2.5 py-1 rounded hover:bg-slate-800 transition-colors {activeMenu === 'help' ? 'bg-slate-800 text-white' : 'text-slate-300'}"
        onclick={() => activeMenu = activeMenu === 'help' ? null : 'help'}
      >
        Help
      </button>
      {#if activeMenu === 'help'}
        <div class="absolute top-full left-0 mt-1 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1.5 z-50 divide-y divide-slate-800 text-xs">
          <div class="py-1">
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={() => { appState.openHelp('user'); closeMenus(); }}
            >
              <span>📖 User Guide</span>
              <span class="text-[10px] text-slate-400 font-mono">F1</span>
            </button>
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center justify-between transition-colors"
              onclick={() => { appState.openHelp('rad'); closeMenus(); }}
            >
              <span>🛠️ RAD Developer Guide</span>
              <span class="text-[10px] text-amber-400 font-mono">Modules</span>
            </button>
          </div>
          <div class="py-1">
            <button
              class="w-full text-left px-3 py-1.5 text-slate-100 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
              onclick={() => { appState.openHelp('about'); closeMenus(); }}
            >
              <span>ℹ️ About R-RAD Studio</span>
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Right Side: Status Indicators & Mode Toggles -->
  <div class="flex items-center gap-3">
    <!-- View Switcher Pills -->
    <div class="flex items-center bg-slate-950 p-0.5 rounded-lg border border-slate-800 text-[11px]">
      <button
        class="px-3 py-1 rounded font-medium transition-all {appState.viewMode === 'data' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
        onclick={() => appState.setView('data')}
      >
        Data Grid
      </button>
      <button
        class="px-3 py-1 rounded font-medium transition-all {appState.viewMode === 'output' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
        onclick={() => appState.setView('output')}
      >
        Output {appState.outputLogs.length > 0 ? `(${appState.outputLogs.length})` : ''}
      </button>
      <button
        class="px-3 py-1 rounded font-medium transition-all {appState.viewMode === 'designer' ? 'bg-amber-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
        onclick={() => appState.setView('designer')}
      >
        Form Designer
      </button>
    </div>

    <!-- Active Dataset Badge -->
    {#if appState.activeDataset}
      <div class="text-[11px] bg-slate-800 border border-slate-700 px-2 py-0.5 rounded flex items-center gap-1.5">
        <span class="text-slate-400">Dataset:</span>
        <span class="font-semibold text-slate-200">{appState.activeDataset.dataset_name}</span>
        <span class="text-[10px] text-slate-500 font-mono">({appState.activeDataset.row_count}×{appState.activeDataset.column_count})</span>
      </div>
    {/if}

    <!-- R Daemon Status Indicator -->
    <div class="flex items-center gap-1.5 text-[11px] font-mono">
      {#if appState.daemonStatus === 'running'}
        <span class="w-2 h-2 rounded-full bg-amber-400 animate-ping"></span>
        <span class="text-amber-300">R Busy...</span>
      {:else if appState.daemonStatus === 'error'}
        <span class="w-2 h-2 rounded-full bg-rose-500"></span>
        <span class="text-rose-400">R Error</span>
      {:else}
        <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
        <span class="text-slate-300">R Ready</span>
      {/if}
    </div>

    <!-- Top Right Corner: R-RAD (in red) with icon.png beside it -->
    <div class="flex items-center gap-1.5 pl-2.5 border-l border-slate-800">
      <span class="font-black tracking-wider text-red-500 font-mono text-sm select-none drop-shadow-sm">R-RAD</span>
      <img src="/icon.png" alt="R-RAD Studio Icon" class="w-5 h-5 rounded object-contain shadow-sm" />
    </div>
  </div>
</div>
