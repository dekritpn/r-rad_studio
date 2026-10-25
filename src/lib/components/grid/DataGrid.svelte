<!-- src/lib/components/grid/DataGrid.svelte -->
<!-- Dual Data Grid: SPSS-Style Virtualized Data View with Row/Cell Editing & Editable Variable View -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import type { VariableMeta, MeasureType } from '../../schemas/dataset';
  import { invoke } from '@tauri-apps/api/core';

  let currentSubTab = $state<'data' | 'variable'>('data');

  // Variable View Inline Edit State
  let editingCell = $state<{ colIndex: number; field: 'label' | 'measure' | 'name' } | null>(null);
  let editValue = $state<string>('');

  // Data View Inline Cell Edit State
  let editingDataCell = $state<{ row: number; colName: string } | null>(null);
  let editDataValue = $state<string>('');

  // Row Header Context Menu
  let rowContextMenu = $state<{ x: number; y: number; rowIndex: number } | null>(null);

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

  // --- SPREADSHEET ROW OPERATIONS ---
  async function insertRow(position?: number) {
    try {
      appState.daemonStatus = 'running';
      const pos = position ?? (appState.selectedRowIndex !== null ? appState.dataOffset + appState.selectedRowIndex + 1 : appState.totalRows + 1);
      const meta: any = await invoke('insert_row', { position: pos });
      appState.setDataset(meta);
      await loadPage(appState.dataOffset);
      await appState.refreshUndoStatus();
      appState.daemonStatus = 'idle';
      appState.showToast(`Inserted new row at line ${pos}`);
    } catch (err) {
      console.error('Failed to insert row:', err);
      appState.daemonStatus = 'error';
      appState.showToast(`Error inserting row: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeRowContextMenu();
    }
  }

  async function deleteSelectedRow(rIdx?: number) {
    const idx = rIdx ?? appState.selectedRowIndex;
    if (idx === null || !appState.activeDataset) return;
    const globalRow = appState.dataOffset + idx + 1;

    try {
      appState.daemonStatus = 'running';
      const meta: any = await invoke('delete_row', { rowIndex: globalRow });
      appState.setDataset(meta);

      const newOffset = Math.min(appState.dataOffset, Math.max(0, meta.row_count - 1));
      await loadPage(newOffset);
      await appState.refreshUndoStatus();
      appState.daemonStatus = 'idle';
      appState.selectedRowIndex = null;
      appState.showToast(`Deleted row ${globalRow}`);
    } catch (err) {
      console.error('Failed to delete row:', err);
      appState.daemonStatus = 'error';
      appState.showToast(`Error deleting row: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      closeRowContextMenu();
    }
  }

  async function copySelectedRow(rIdx?: number) {
    const idx = rIdx ?? appState.selectedRowIndex;
    if (idx === null || !appState.dataRows[idx] || !appState.activeDataset) return;
    const row = appState.dataRows[idx];
    const tsv = appState.activeDataset.variables.map(v => row[v.name] ?? '').join('\t');
    try {
      await navigator.clipboard.writeText(tsv);
      appState.showToast(`Copied row ${appState.dataOffset + idx + 1} to clipboard (TSV)`);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    } finally {
      closeRowContextMenu();
    }
  }

  async function copyActiveCell() {
    if (!appState.activeDataCell || !appState.dataRows[appState.activeDataCell.row]) {
      if (appState.selectedRowIndex !== null) {
        await copySelectedRow(appState.selectedRowIndex);
      }
      return;
    }
    const row = appState.dataRows[appState.activeDataCell.row];
    const val = String(row[appState.activeDataCell.colName] ?? '');
    try {
      await navigator.clipboard.writeText(val);
      appState.showToast('Copied cell value to clipboard');
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  async function pasteFromClipboard() {
    if (!appState.activeDataset) return;
    try {
      const text = await navigator.clipboard.readText();
      if (!text || !text.trim()) {
        appState.showToast('Clipboard is empty');
        return;
      }

      const lines = text.trim().split(/\r?\n/).filter(l => l.length > 0);
      const matrix = lines.map(line => line.split('\t'));

      const startRow = appState.activeDataCell
        ? appState.dataOffset + appState.activeDataCell.row + 1
        : (appState.selectedRowIndex !== null ? appState.dataOffset + appState.selectedRowIndex + 1 : 1);

      const startCol = appState.activeDataCell
        ? appState.activeDataCell.colName
        : (appState.activeDataset.variables[0]?.name || '');

      appState.daemonStatus = 'running';
      const meta: any = await invoke('batch_update_cells', {
        startRow,
        startCol,
        matrix,
      });
      appState.setDataset(meta);
      await loadPage(appState.dataOffset);
      await appState.refreshUndoStatus();
      appState.daemonStatus = 'idle';
      appState.showToast(`Pasted ${matrix.length} row(s) into dataset`);
    } catch (err) {
      console.error('Failed to paste from clipboard:', err);
      appState.daemonStatus = 'error';
      appState.showToast(`Paste error: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  function openRowContextMenu(e: MouseEvent, rIdx: number) {
    e.preventDefault();
    appState.selectedRowIndex = rIdx;
    rowContextMenu = { x: e.clientX, y: e.clientY, rowIndex: rIdx };
  }

  function closeRowContextMenu() {
    rowContextMenu = null;
  }

  // --- SPREADSHEET INLINE DATA CELL EDITING ---
  function startEditDataCell(rIdx: number, colName: string, currentVal: any) {
    editingDataCell = { row: rIdx, colName };
    editDataValue = currentVal === null || currentVal === undefined ? '' : String(currentVal);
    appState.activeDataCell = { row: rIdx, colName };
    appState.selectedRowIndex = rIdx;
  }

  async function saveEditDataCell(rIdx: number, colName: string) {
    if (!editingDataCell || editingDataCell.row !== rIdx || editingDataCell.colName !== colName) return;
    const globalRow = appState.dataOffset + rIdx + 1;
    const val = editDataValue;
    editingDataCell = null;

    try {
      const meta: any = await invoke('update_cell', {
        rowIndex: globalRow,
        colName,
        value: val,
      });
      appState.setDataset(meta);
      if (appState.dataRows[rIdx]) {
        appState.dataRows[rIdx][colName] = val === '' ? null : (isNaN(Number(val)) ? val : Number(val));
      }
      await appState.refreshUndoStatus();
    } catch (err) {
      console.error('Failed to update cell:', err);
      appState.showToast(`Error updating cell: ${err instanceof Error ? err.message : String(err)}`);
      await loadPage(appState.dataOffset);
    }
  }

  function cancelEditDataCell() {
    editingDataCell = null;
  }

  async function saveEditDataCellAndNext(rIdx: number, cIdx: number) {
    if (!editingDataCell || !appState.activeDataset) return;
    const curColName = editingDataCell.colName;
    await saveEditDataCell(rIdx, curColName);

    const vars = appState.activeDataset.variables;
    if (cIdx + 1 < vars.length) {
      const nextVar = vars[cIdx + 1];
      startEditDataCell(rIdx, nextVar.name, appState.dataRows[rIdx]?.[nextVar.name]);
    } else if (rIdx + 1 < appState.dataRows.length) {
      const nextVar = vars[0];
      startEditDataCell(rIdx + 1, nextVar.name, appState.dataRows[rIdx + 1]?.[nextVar.name]);
    }
  }

  function handleDataGridKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT')) {
      return;
    }

    if (currentSubTab !== 'data') return;

    if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
      copyActiveCell();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
      pasteFromClipboard();
    } else if (e.key === 'Delete' && appState.selectedRowIndex !== null) {
      deleteSelectedRow();
    } else if (e.key === 'F2' && appState.activeDataCell) {
      const { row, colName } = appState.activeDataCell;
      startEditDataCell(row, colName, appState.dataRows[row]?.[colName]);
    }
  }

  // --- VARIABLE VIEW EDITING ---
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

<svelte:window onclick={closeRowContextMenu} onkeydown={handleDataGridKeydown} />

<div class="flex flex-col h-full bg-slate-900 text-slate-100 select-none font-data-custom">
  <!-- Data View Action Bar (Spreadsheet Quick Toolbar) -->
  {#if currentSubTab === 'data'}
    <div class="h-8 bg-slate-950/90 border-b border-slate-800 px-3 flex items-center justify-between text-xs select-none">
      <div class="flex items-center gap-1.5">
        <!-- Undo & Redo Quick Actions -->
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] flex items-center gap-1 transition-colors disabled:opacity-40 disabled:hover:bg-slate-800 disabled:cursor-not-allowed"
          onclick={() => appState.undo()}
          disabled={!appState.canUndo}
          title="Undo last data change (Ctrl+Z)"
        >
          <span>↶ Undo</span>
          {#if appState.undoCount > 0}
            <span class="text-[9px] bg-indigo-900/80 text-indigo-300 px-1 rounded-full font-mono">{appState.undoCount}</span>
          {/if}
        </button>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] flex items-center gap-1 transition-colors disabled:opacity-40 disabled:hover:bg-slate-800 disabled:cursor-not-allowed"
          onclick={() => appState.redo()}
          disabled={!appState.canRedo}
          title="Redo data change (Ctrl+Y)"
        >
          <span>↷ Redo</span>
          {#if appState.redoCount > 0}
            <span class="text-[9px] bg-indigo-900/80 text-indigo-300 px-1 rounded-full font-mono">{appState.redoCount}</span>
          {/if}
        </button>
        <div class="h-3.5 w-px bg-slate-800 mx-1"></div>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] flex items-center gap-1 transition-colors"
          onclick={() => insertRow()}
          title="Insert new row / case at end or position"
        >
          <span>➕ Insert Row</span>
        </button>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-rose-900/60 text-slate-200 hover:text-rose-200 text-[11px] flex items-center gap-1 transition-colors disabled:opacity-40"
          onclick={() => deleteSelectedRow()}
          disabled={appState.selectedRowIndex === null}
          title="Delete selected row / case"
        >
          <span>🗑️ Delete Row</span>
        </button>
        <div class="h-3.5 w-px bg-slate-800 mx-1"></div>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] flex items-center gap-1 transition-colors"
          onclick={copyActiveCell}
          title="Copy selected cell or row (Ctrl+C)"
        >
          <span>📋 Copy</span>
        </button>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] flex items-center gap-1 transition-colors"
          onclick={pasteFromClipboard}
          title="Paste from clipboard (Ctrl+V)"
        >
          <span>📥 Paste</span>
        </button>
        <div class="h-3.5 w-px bg-slate-800 mx-1"></div>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-indigo-900/60 text-slate-200 hover:text-indigo-200 text-[11px] flex items-center gap-1 transition-colors"
          onclick={() => appState.openFindReplace('find')}
          title="Find in dataset (Ctrl+F)"
        >
          <span>🔍 Find...</span>
        </button>
        <button
          type="button"
          class="px-2 py-0.5 rounded bg-slate-800 hover:bg-indigo-900/60 text-slate-200 hover:text-indigo-200 text-[11px] flex items-center gap-1 transition-colors"
          onclick={() => appState.openFindReplace('replace')}
          title="Find & Replace in dataset (Ctrl+H)"
        >
          <span>🔄 Replace...</span>
        </button>
      </div>

      <div class="text-[11px] text-slate-500 font-mono">
        {#if appState.activeDataCell}
          Row {appState.dataOffset + appState.activeDataCell.row + 1}, Col: {appState.activeDataCell.colName} (Double-click to edit)
        {:else if appState.selectedRowIndex !== null}
          Row {appState.dataOffset + appState.selectedRowIndex + 1} selected
        {/if}
      </div>
    </div>
  {/if}

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
              <tr class="hover:bg-slate-800/40 transition-colors {appState.selectedRowIndex === rIdx ? 'bg-indigo-950/40' : ''}">
                <!-- Row Number Header (Selectable & Context Menu) -->
                <td
                  class="px-2 py-1.5 text-center text-[10px] select-none cursor-pointer border-r border-slate-800 transition-colors {appState.selectedRowIndex === rIdx ? 'bg-indigo-600 text-white font-bold' : 'bg-slate-900 text-slate-500 hover:bg-slate-800 hover:text-slate-200'}"
                  onclick={() => appState.selectedRowIndex = rIdx}
                  oncontextmenu={(e) => openRowContextMenu(e, rIdx)}
                  title="Click to select row, right-click for options"
                >
                  {appState.dataOffset + rIdx + 1}
                </td>
                <!-- Columns -->
                {#each appState.activeDataset.variables as v, cIdx}
                  <td
                    id={`cell_${rIdx}_${v.name}`}
                    class="px-2 py-1 border-r border-slate-800/60 truncate cursor-pointer {appState.activeDataCell?.row === rIdx && appState.activeDataCell?.colName === v.name ? 'ring-2 ring-indigo-500 ring-inset bg-indigo-900/40' : ''} {v.align === 'right' ? 'text-right' : 'text-left'}"
                    onclick={() => {
                      appState.activeDataCell = { row: rIdx, colName: v.name };
                      appState.selectedRowIndex = rIdx;
                    }}
                    ondblclick={() => startEditDataCell(rIdx, v.name, row[v.name])}
                  >
                    {#if editingDataCell?.row === rIdx && editingDataCell?.colName === v.name}
                      <input
                        type="text"
                        bind:value={editDataValue}
                        class="w-full bg-slate-950 border border-indigo-400 rounded px-1.5 py-0.5 text-xs text-white font-mono focus:outline-none"
                        autofocus
                        onkeydown={(e) => {
                          if (e.key === 'Enter') saveEditDataCell(rIdx, v.name);
                          if (e.key === 'Escape') cancelEditDataCell();
                          if (e.key === 'Tab') {
                            e.preventDefault();
                            saveEditDataCellAndNext(rIdx, cIdx);
                          }
                        }}
                        onblur={() => saveEditDataCell(rIdx, v.name)}
                      />
                    {:else}
                      {#if row[v.name] === null || row[v.name] === undefined}
                        <span class="text-slate-600 font-sans">.</span>
                      {:else if typeof row[v.name] === 'number'}
                        <span class="{appState.findHighlightQuery && String(row[v.name]).includes(appState.findHighlightQuery) ? 'bg-amber-500/30 text-amber-200 px-0.5 rounded font-bold' : ''}">
                          {(row[v.name] as number).toFixed(v.decimals)}
                        </span>
                      {:else}
                        <span class="{appState.findHighlightQuery && String(row[v.name]).includes(appState.findHighlightQuery) ? 'bg-amber-500/30 text-amber-200 px-0.5 rounded font-bold' : ''}">
                          {String(row[v.name])}
                        </span>
                      {/if}
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
                      role="button"
                      tabindex="0"
                      onkeydown={(e) => { if (e.key === 'Enter') startEdit(idx, 'name', v.name); }}
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
                  {:else}
                    <div
                      class="text-slate-200 min-h-[18px] flex items-center justify-between group"
                      onclick={() => startEdit(idx, 'label', v.label)}
                      title="Click to edit variable label"
                      role="button"
                      tabindex="0"
                      onkeydown={(e) => { if (e.key === 'Enter') startEdit(idx, 'label', v.label); }}
                    >
                      <span>{v.label || '<none>'}</span>
                      <span class="text-[10px] text-slate-500 opacity-0 group-hover:opacity-100 transition-opacity">✏️</span>
                    </div>
                  {/if}
                </td>
                <!-- Values -->
                <td class="px-3 py-2 text-slate-400 border-r border-slate-800 text-[11px] truncate">
                  {#if v.values && v.values.length > 0}
                    {v.values.map(val => `${val.value} = ${val.label}`).join(', ')}
                  {:else}
                    None
                  {/if}
                </td>
                <!-- Missing -->
                <td class="px-3 py-2 text-slate-400 border-r border-slate-800 text-[11px]">
                  {v.missing || 'None'}
                </td>
                <!-- Align -->
                <td class="px-3 py-2 text-center text-slate-400 border-r border-slate-800 capitalize">
                  {v.align}
                </td>
                <!-- Measure -->
                <td class="px-3 py-2 border-r border-slate-800">
                  {#if editingCell?.colIndex === idx && editingCell.field === 'measure'}
                    <select
                      bind:value={editValue}
                      class="bg-slate-900 border border-indigo-500 rounded px-1.5 py-0.5 text-xs text-white focus:outline-none"
                      onchange={() => saveEdit(idx)}
                      onblur={() => saveEdit(idx)}
                    >
                      <option value="scale">Scale</option>
                      <option value="ordinal">Ordinal</option>
                      <option value="nominal">Nominal</option>
                    </select>
                  {:else}
                    <div
                      class="flex items-center gap-1.5 cursor-pointer hover:bg-slate-750 px-1 py-0.5 rounded transition-colors"
                      onclick={() => startEdit(idx, 'measure', v.measure)}
                      role="button"
                      tabindex="0"
                      onkeydown={(e) => { if (e.key === 'Enter') startEdit(idx, 'measure', v.measure); }}
                    >
                      {#if v.measure === 'scale'}
                        <span class="text-amber-400 font-bold">📐</span>
                        <span class="capitalize text-slate-200">Scale</span>
                      {:else if v.measure === 'ordinal'}
                        <span class="text-sky-400 font-bold">📊</span>
                        <span class="capitalize text-slate-200">Ordinal</span>
                      {:else}
                        <span class="text-emerald-400 font-bold">⚪</span>
                        <span class="capitalize text-slate-200">Nominal</span>
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

  <!-- Row Context Menu -->
  {#if rowContextMenu}
    <div
      class="fixed z-50 bg-slate-900 border border-slate-700 rounded-lg shadow-2xl py-1 text-xs text-slate-200 w-48 divide-y divide-slate-800"
      style="top: {rowContextMenu.y}px; left: {rowContextMenu.x}px;"
      onclick={(e) => e.stopPropagation()}
      role="menu"
      tabindex="-1"
    >
      <div class="px-3 py-1 text-[10px] uppercase font-bold text-slate-400">
        Row {appState.dataOffset + rowContextMenu.rowIndex + 1}
      </div>
      <div class="py-1">
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
          onclick={() => insertRow(appState.dataOffset + rowContextMenu!.rowIndex + 1)}
        >
          <span>➕ Insert Row Above</span>
        </button>
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
          onclick={() => insertRow(appState.dataOffset + rowContextMenu!.rowIndex + 2)}
        >
          <span>➕ Insert Row Below</span>
        </button>
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 hover:bg-indigo-600 hover:text-white flex items-center gap-2 transition-colors"
          onclick={() => copySelectedRow(rowContextMenu!.rowIndex)}
        >
          <span>📋 Copy Row (TSV)</span>
        </button>
      </div>
      <div class="py-1">
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 hover:bg-rose-600 text-rose-300 hover:text-white flex items-center gap-2 transition-colors"
          onclick={() => deleteSelectedRow(rowContextMenu!.rowIndex)}
        >
          <span>🗑️ Delete Row</span>
        </button>
      </div>
    </div>
  {/if}

  <!-- Bottom Workbench Sub-Tabs: Data View vs Variable View -->
  <div class="h-9 bg-slate-950 border-t border-slate-800 flex items-center justify-between px-3 z-10 select-none">
    <div class="flex items-center gap-1 -mb-px">
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
