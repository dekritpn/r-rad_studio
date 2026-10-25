<!-- src/lib/components/grid/DataGrid.svelte -->
<!-- Dual Data Grid: SPSS-Style Virtualized Data View and Editable Variable View -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import type { VariableMeta, MeasureType } from '../../schemas/dataset';
  import { invoke } from '@tauri-apps/api/core';

  let currentSubTab = $state<'data' | 'variable'>('data');
  let editingCell = $state<{ colIndex: number; field: 'label' | 'measure' | 'name' } | null>(null);
  let editValue = $state<string>('');
  let activeCell = $state<{ row: number; col: number } | null>(null);

  // Pagination navigation
  const currentPage = $derived(Math.floor(appState.dataOffset / appState.dataLimit) + 1);
  const totalPages = $derived(Math.max(1, Math.ceil(appState.totalRows / appState.dataLimit)));

  async function loadPage(offset: number) {
    if (appState.isLoadingData) return;
    appState.isLoadingData = true;
    try {
      const slice: any = await invoke('get_data_page', {
        offset,
        limit: appState.dataLimit,
      });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
    } catch (err) {
      console.error('Failed to load page slice:', err);
    } finally {
      appState.isLoadingData = false;
    }
  }

  function nextPage() {
    if (appState.dataOffset + appState.dataLimit < appState.totalRows) {
      loadPage(appState.dataOffset + appState.dataLimit);
    }
  }

  function prevPage() {
    if (appState.dataOffset >= appState.dataLimit) {
      loadPage(appState.dataOffset - appState.dataLimit);
    }
  }

  function startEdit(colIndex: number, field: 'label' | 'measure' | 'name', currentVal: string) {
    editingCell = { colIndex, field };
    editValue = currentVal;
  }

  async function saveEdit(colIndex: number) {
    if (!editingCell || !appState.activeDataset) return;
    const variable = appState.activeDataset.variables[colIndex];
    if (!variable) return;

    const updates: Record<string, string> = {};
    if (editingCell.field === 'label') updates.label = editValue;
    if (editingCell.field === 'measure') updates.measure = editValue;
    if (editingCell.field === 'name') updates.name = editValue;

    try {
      const updatedVars: VariableMeta[] = await invoke('update_variable_metadata', {
        colName: variable.name,
        updates,
      });
      appState.updateVariables(updatedVars);
    } catch (err) {
      console.error('Failed to update variable metadata:', err);
    } finally {
      editingCell = null;
    }
  }

  function cancelEdit() {
    editingCell = null;
  }

  async function saveNameEdit(colIndex: number, oldName: string) {
    if (!editingCell || !appState.activeDataset) return;
    const newName = editValue.trim();
    if (!newName || newName === oldName) {
      editingCell = null;
      return;
    }
    try {
      const updatedVars: VariableMeta[] = await invoke('rename_variable', {
        oldName,
        newName,
      });
      appState.updateVariables(updatedVars);
      // Reload current page slice so Data View headers update immediately
      await loadPage(appState.dataOffset);
      appState.showToast(`Renamed variable '${oldName}' to '${newName}'`);
    } catch (err) {
      console.error('Failed to rename variable:', err);
      appState.showToast(`Error renaming variable: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      editingCell = null;
    }
  }

  async function addNewVariable() {
    try {
      const nextNum = (appState.activeDataset?.column_count ?? 0) + 1;
      const newVarName = `VAR${String(nextNum).padStart(5, '0')}`;
      const meta: any = await invoke('add_variable', {
        name: newVarName,
        measure: 'scale',
        label: newVarName,
      });
      appState.setDataset(meta);
      await loadPage(appState.dataOffset);
      appState.showToast(`Added new blank variable '${newVarName}'`);
    } catch (err) {
      console.error('Failed to add variable:', err);
      appState.showToast(`Error adding variable: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
</script>

<div class="flex flex-col h-full bg-slate-900 text-slate-100 select-none">
  <!-- Grid Content Area -->
  <div class="flex-1 overflow-auto relative">
    {#if currentSubTab === 'data'}
      <!-- DATA VIEW -->
      {#if !appState.activeDataset || appState.activeDataset.variables.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-slate-400">
          <svg class="w-16 h-16 mb-4 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 10h18M3 14h18m-9-4v8m-7 4h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <p class="text-base font-medium">No Dataset Active</p>
          <p class="text-xs text-slate-500 mt-1">Load a dataset from File &rarr; Open Dataset or sample catalogs</p>
        </div>
      {:else}
        <table class="w-full text-xs border-collapse">
          <!-- Sticky Header -->
          <thead class="sticky top-0 bg-slate-800 text-slate-200 z-10 shadow-sm border-b border-slate-700">
            <tr>
              <!-- Row Number Corner Header -->
              <th class="w-14 px-2 py-2 text-center font-mono text-[11px] font-semibold text-slate-400 bg-slate-800 border-r border-slate-700">#</th>
              {#each appState.activeDataset.variables as v, cIdx}
                <th
                  class="px-3 py-2 text-left font-medium border-r border-slate-700 min-w-[120px] max-w-[220px] hover:bg-slate-700/60 transition-colors cursor-pointer"
                  title="{v.name}{v.label ? ` (${v.label})` : ''} — Measurement: {v.measure}"
                >
                  <div class="flex items-center gap-1.5 truncate">
                    <!-- Measurement Icon -->
                    {#if v.measure === 'scale'}
                      <span class="text-amber-400 font-bold" title="Scale (Numeric)">📐</span>
                    {:else if v.measure === 'ordinal'}
                      <span class="text-sky-400 font-bold" title="Ordinal">📊</span>
                    {:else}
                      <span class="text-emerald-400 font-bold" title="Nominal">⚪</span>
                    {/if}
                    <span class="font-mono font-semibold truncate text-slate-100">{v.name}</span>
                  </div>
                  {#if v.label}
                    <div class="text-[10px] text-slate-400 truncate font-normal mt-0.5">{v.label}</div>
                  {/if}
                </th>
              {/each}
            </tr>
          </thead>
          <!-- Data Rows -->
          <tbody class="divide-y divide-slate-800/80 font-mono">
            {#each appState.dataRows as row, rIdx}
              <tr class="hover:bg-slate-800/40 transition-colors {activeCell?.row === rIdx ? 'bg-indigo-950/30' : ''}">
                <!-- Row Number -->
                <td class="px-2 py-1.5 text-center text-[10px] text-slate-500 bg-slate-900 border-r border-slate-800 select-none">
                  {appState.dataOffset + rIdx + 1}
                </td>
                <!-- Columns -->
                {#each appState.activeDataset.variables as v, cIdx}
                  <td
                    class="px-3 py-1.5 border-r border-slate-800/60 truncate cursor-pointer {activeCell?.row === rIdx && activeCell?.col === cIdx ? 'ring-2 ring-indigo-500 ring-inset bg-indigo-900/40' : ''} {v.align === 'right' ? 'text-right' : 'text-left'}"
                    onclick={() => activeCell = { row: rIdx, col: cIdx }}
                  >
                    {#if row[v.name] === null || row[v.name] === undefined}
                      <span class="text-slate-600 font-sans">.</span>
                    {:else if typeof row[v.name] === 'number'}
                      {(row[v.name] as number).toFixed(v.decimals)}
                    {:else}
                      {String(row[v.name])}
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {:else}
      <!-- VARIABLE VIEW -->
      {#if !appState.activeDataset || appState.activeDataset.variables.length === 0}
        <div class="flex items-center justify-center h-full text-slate-500 text-sm">
          No variables available
        </div>
      {:else}
        <table class="w-full text-xs border-collapse">
          <thead class="sticky top-0 bg-slate-800 text-slate-200 z-10 shadow-sm border-b border-slate-700">
            <tr>
              <th class="w-12 px-2 py-2 text-center text-slate-400 font-semibold border-r border-slate-700">#</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 w-36">Name</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 w-24">Type</th>
              <th class="px-3 py-2 text-center font-semibold border-r border-slate-700 w-16">Width</th>
              <th class="px-3 py-2 text-center font-semibold border-r border-slate-700 w-16">Decimals</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 min-w-[200px]">Label (Click to edit)</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 w-44">Values</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 w-24">Missing</th>
              <th class="px-3 py-2 text-center font-semibold border-r border-slate-700 w-20">Align</th>
              <th class="px-3 py-2 text-left font-semibold border-r border-slate-700 w-32">Measure</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800">
            {#each appState.activeDataset.variables as v, idx}
              <tr class="hover:bg-slate-800/50 transition-colors">
                <td class="px-2 py-2 text-center text-[11px] text-slate-500 bg-slate-900 border-r border-slate-800 font-mono">
                  {idx + 1}
                </td>
                <!-- Name (Inline Editable) -->
                <td class="px-3 py-2 font-mono font-semibold border-r border-slate-800 cursor-pointer hover:bg-slate-750 transition-colors">
                  {#if editingCell?.colIndex === idx && editingCell.field === 'name'}
                    <input
                      type="text"
                      bind:value={editValue}
                      class="bg-slate-900 border border-indigo-500 rounded px-2 py-1 text-xs text-white font-mono focus:outline-none w-full"
                      autofocus
                      onkeydown={(e) => {
                        if (e.key === 'Enter') saveNameEdit(idx, v.name);
                        if (e.key === 'Escape') cancelEdit();
                      }}
                      onblur={() => saveNameEdit(idx, v.name)}
                    />
                  {:else}
                    <div
                      class="text-slate-100 min-h-[18px] flex items-center justify-between group"
                      onclick={() => startEdit(idx, 'name', v.name)}
                      title="Click to rename variable"
                    >
                      <span>{v.name}</span>
                      <span class="text-[10px] text-slate-500 opacity-0 group-hover:opacity-100 transition-opacity">✏️</span>
                    </div>
                  {/if}
                </td>
                <!-- Type -->
                <td class="px-3 py-2 text-slate-300 border-r border-slate-800">
                  {v.type}
                </td>
                <!-- Width -->
                <td class="px-3 py-2 text-center text-slate-400 border-r border-slate-800 font-mono">
                  {v.width}
                </td>
                <!-- Decimals -->
                <td class="px-3 py-2 text-center text-slate-400 border-r border-slate-800 font-mono">
                  {v.decimals}
                </td>
                <!-- Label (Inline Editable) -->
                <td class="px-3 py-2 border-r border-slate-800 cursor-pointer hover:bg-slate-750 transition-colors">
                  {#if editingCell?.colIndex === idx && editingCell.field === 'label'}
                    <div class="flex items-center gap-1">
                      <input
                        type="text"
                        bind:value={editValue}
                        class="bg-slate-900 border border-indigo-500 rounded px-2 py-1 text-xs text-white focus:outline-none w-full"
                        autofocus
                        onkeydown={(e) => {
                          if (e.key === 'Enter') saveEdit(idx);
                          if (e.key === 'Escape') cancelEdit();
                        }}
                        onblur={() => saveEdit(idx)}
                      />
                    </div>
                  {:else}
                    <div
                      class="text-slate-100 min-h-[18px]"
                      onclick={() => startEdit(idx, 'label', v.label)}
                    >
                      {v.label || '<none>'}
                    </div>
                  {/if}
                </td>
                <!-- Values -->
                <td class="px-3 py-2 border-r border-slate-800 text-slate-400 truncate max-w-xs font-mono text-[11px]">
                  {#if v.values && v.values.length > 0}
                    {`{${v.values.map(val => `${val.value}=${val.label}`).slice(0, 3).join(', ')}${v.values.length > 3 ? '...' : ''}}`}
                  {:else}
                    None
                  {/if}
                </td>
                <!-- Missing -->
                <td class="px-3 py-2 border-r border-slate-800 text-slate-400">
                  {v.missing || 'None'}
                </td>
                <!-- Align -->
                <td class="px-3 py-2 text-center text-slate-400 border-r border-slate-800">
                  {v.align}
                </td>
                <!-- Measure (Editable Dropdown) -->
                <td class="px-3 py-2 border-r border-slate-800">
                  {#if editingCell?.colIndex === idx && editingCell.field === 'measure'}
                    <select
                      bind:value={editValue}
                      class="bg-slate-900 border border-indigo-500 rounded px-1.5 py-1 text-xs text-white focus:outline-none"
                      onchange={() => saveEdit(idx)}
                      onblur={() => saveEdit(idx)}
                      autofocus
                    >
                      <option value="scale">Scale</option>
                      <option value="ordinal">Ordinal</option>
                      <option value="nominal">Nominal</option>
                    </select>
                  {:else}
                    <div
                      class="flex items-center gap-1.5 cursor-pointer hover:bg-slate-750 px-1 py-0.5 rounded transition-colors"
                      onclick={() => startEdit(idx, 'measure', v.measure)}
                    >
                      {#if v.measure === 'scale'}
                        <span class="text-amber-400 font-bold">📐</span>
                        <span>Scale</span>
                      {:else if v.measure === 'ordinal'}
                        <span class="text-sky-400 font-bold">📊</span>
                        <span>Ordinal</span>
                      {:else}
                        <span class="text-emerald-400 font-bold">⚪</span>
                        <span>Nominal</span>
                      {/if}
                    </div>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </div>

  <!-- Bottom SPSS-Style Dual Tabs & Pagination Toolbar -->
  <div class="h-10 bg-slate-950 border-t border-slate-800 flex items-center justify-between px-3 z-20">
    <!-- SPSS Bottom Dual Tabs -->
    <div class="flex items-center gap-1">
      <button
        class="px-4 py-1.5 text-xs font-semibold rounded-t border-t border-l border-r transition-all flex items-center gap-1.5 {currentSubTab === 'data' ? 'bg-slate-800 text-white border-slate-700 shadow-sm' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'}"
        onclick={() => currentSubTab = 'data'}
      >
        <svg class="w-3.5 h-3.5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M3 14h18m-9-4v8m-7 4h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        Data View
      </button>
      <button
        class="px-4 py-1.5 text-xs font-semibold rounded-t border-t border-l border-r transition-all flex items-center gap-1.5 {currentSubTab === 'variable' ? 'bg-slate-800 text-white border-slate-700 shadow-sm' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'}"
        onclick={() => currentSubTab = 'variable'}
      >
        <svg class="w-3.5 h-3.5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
        </svg>
        Variable View
      </button>
    </div>

    <!-- Pagination & Row Stats -->
    {#if currentSubTab === 'data' && appState.totalRows > 0}
      <div class="flex items-center gap-3 text-xs text-slate-400 font-mono">
        <span>Rows: <strong class="text-slate-200">{appState.dataOffset + 1}–{Math.min(appState.totalRows, appState.dataOffset + appState.dataLimit)}</strong> of <strong class="text-slate-200">{appState.totalRows}</strong></span>
        <div class="flex items-center gap-1">
          <button
            class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-xs transition-colors"
            onclick={prevPage}
            disabled={appState.dataOffset === 0 || appState.isLoadingData}
          >
            &larr; Prev
          </button>
          <span class="px-1 text-slate-500 font-sans">{currentPage} / {totalPages}</span>
          <button
            class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-xs transition-colors"
            onclick={nextPage}
            disabled={appState.dataOffset + appState.dataLimit >= appState.totalRows || appState.isLoadingData}
          >
            Next &rarr;
          </button>
        </div>
      </div>
    {:else if currentSubTab === 'variable'}
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="px-3 py-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-xs font-semibold shadow transition-colors flex items-center gap-1.5"
          onclick={addNewVariable}
          title="Add a new blank variable column to active dataset"
        >
          <span>➕ Add Variable</span>
        </button>
      </div>
    {/if}
  </div>
</div>
