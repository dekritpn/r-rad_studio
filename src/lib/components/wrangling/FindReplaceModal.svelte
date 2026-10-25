<!-- src/lib/components/wrangling/FindReplaceModal.svelte -->
<!-- SPSS/Excel-Style Find & Replace Dialog for Data Grid -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let activeTab = $state<'find' | 'replace'>('find');
  let findText = $state<string>('');
  let replaceText = $state<string>('');
  let targetCol = $state<string>('all');
  let matchEntireCell = $state<boolean>(true);
  let matchCase = $state<boolean>(false);
  let useRegex = $state<boolean>(false);

  let statusMessage = $state<string | null>(null);
  let statusType = $state<'info' | 'success' | 'error'>('info');
  let isProcessing = $state<boolean>(false);

  $effect(() => {
    if (appState.isFindReplaceOpen) {
      activeTab = appState.findReplaceInitialTab;
      statusMessage = null;
    }
  });

  function close() {
    appState.closeFindReplace();
    appState.findHighlightQuery = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'Enter' && !isProcessing) {
      if (activeTab === 'find') {
        findNext();
      } else {
        handleReplace();
      }
    }
  }

  // Find Next in current page
  function findNext() {
    if (!findText) {
      statusMessage = 'Please enter a search term';
      statusType = 'error';
      return;
    }

    appState.findHighlightQuery = findText;

    if (!appState.dataRows || appState.dataRows.length === 0) {
      statusMessage = 'No data rows available to search';
      statusType = 'info';
      return;
    }

    const cols = (targetCol === 'all' || !targetCol)
      ? (appState.activeDataset?.variables.map(v => v.name) || [])
      : [targetCol];

    let query = findText;
    let foundCell: { row: number; colName: string } | null = null;

    // Start looking from current active cell or start
    const startRow = appState.activeDataCell ? appState.activeDataCell.row : 0;
    const startColIdx = appState.activeDataCell
      ? cols.indexOf(appState.activeDataCell.colName)
      : -1;

    // Check cells
    for (let r = 0; r < appState.dataRows.length; r++) {
      const rowIndex = (startRow + r) % appState.dataRows.length;
      const row = appState.dataRows[rowIndex];

      for (let c = 0; c < cols.length; c++) {
        // If same row as start, skip up to current col
        if (r === 0 && startColIdx >= 0 && c <= startColIdx) {
          continue;
        }

        const colName = cols[c];
        const rawVal = row[colName];
        if (rawVal === undefined || rawVal === null) continue;

        const valStr = String(rawVal);
        let isMatch = false;

        if (useRegex) {
          try {
            const flags = matchCase ? '' : 'i';
            const pat = matchEntireCell ? `^${query}$` : query;
            const re = new RegExp(pat, flags);
            isMatch = re.test(valStr);
          } catch {
            statusMessage = 'Invalid regular expression';
            statusType = 'error';
            return;
          }
        } else {
          if (matchEntireCell) {
            isMatch = matchCase
              ? valStr === query
              : valStr.toLowerCase() === query.toLowerCase();
          } else {
            isMatch = matchCase
              ? valStr.includes(query)
              : valStr.toLowerCase().includes(query.toLowerCase());
          }
        }

        if (isMatch) {
          foundCell = { row: rowIndex, colName };
          break;
        }
      }

      if (foundCell) break;
    }

    if (foundCell) {
      appState.activeDataCell = foundCell;
      appState.selectedRowIndex = foundCell.row;
      statusMessage = `Found match at Row ${appState.dataOffset + foundCell.row + 1}, Variable '${foundCell.colName}'`;
      statusType = 'success';

      // Scroll cell into view
      const cellId = `cell_${foundCell.row}_${foundCell.colName}`;
      const el = document.getElementById(cellId);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
      }
    } else {
      statusMessage = `No match found for '${findText}' on this page`;
      statusType = 'info';
    }
  }

  // Replace active match
  async function handleReplace() {
    if (!appState.activeDataCell) {
      findNext();
      return;
    }

    const { row, colName } = appState.activeDataCell;
    const globalRow = appState.dataOffset + row + 1;

    try {
      isProcessing = true;
      const meta: any = await invoke('update_cell', {
        rowIndex: globalRow,
        colName,
        value: replaceText,
      });
      appState.setDataset(meta);

      // Reload data page
      const slice: any = await invoke('get_data_page', {
        offset: appState.dataOffset,
        limit: appState.dataLimit,
      });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
      await appState.refreshUndoStatus();

      statusMessage = `Replaced at Row ${globalRow}, Variable '${colName}'`;
      statusType = 'success';

      // Find next
      findNext();
    } catch (err) {
      console.error('Failed to replace cell:', err);
      statusMessage = `Error replacing: ${err instanceof Error ? err.message : String(err)}`;
      statusType = 'error';
    } finally {
      isProcessing = false;
    }
  }

  // Replace All in dataset
  async function handleReplaceAll() {
    if (!findText) {
      statusMessage = 'Please enter a search term';
      statusType = 'error';
      return;
    }

    try {
      isProcessing = true;
      appState.daemonStatus = 'running';

      const res: any = await invoke('find_replace', {
        findText,
        replaceText,
        colName: targetCol === 'all' ? null : targetCol,
        matchEntireCell,
        matchCase,
        useRegex,
      });

      const count = res.replaced_count ?? 0;
      appState.setDataset({
        dataset_name: res.dataset_name,
        row_count: res.row_count,
        column_count: res.column_count,
        variables: res.variables,
      });

      // Reload data page
      const slice: any = await invoke('get_data_page', {
        offset: appState.dataOffset,
        limit: appState.dataLimit,
      });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);
      await appState.refreshUndoStatus();
      appState.daemonStatus = 'idle';

      statusMessage = `Done! Replaced ${count} occurrence${count === 1 ? '' : 's'}.`;
      statusType = 'success';
      appState.showToast(`Find & Replace: Replaced ${count} values`);
    } catch (err) {
      console.error('Failed to replace all:', err);
      appState.daemonStatus = 'error';
      statusMessage = `Error in Replace All: ${err instanceof Error ? err.message : String(err)}`;
      statusType = 'error';
    } finally {
      isProcessing = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if appState.isFindReplaceOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div
      class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col w-full max-w-md overflow-hidden text-slate-100 font-sans"
      role="dialog"
      aria-modal="true"
    >
      <!-- Dialog Header -->
      <div class="px-4 py-3 border-b border-slate-800 bg-slate-950 flex items-center justify-between select-none">
        <div class="flex items-center gap-2">
          <span class="text-base">🔍</span>
          <h2 class="text-xs font-semibold text-white tracking-wide">
            {activeTab === 'find' ? 'Find in Dataset' : 'Find & Replace in Dataset'}
          </h2>
        </div>

        <!-- Mode Toggle Tabs -->
        <div class="flex items-center bg-slate-900 border border-slate-800 rounded p-0.5 text-xs">
          <button
            type="button"
            class="px-2.5 py-1 rounded text-[11px] font-medium transition-all {activeTab === 'find' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => activeTab = 'find'}
          >
            Find
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded text-[11px] font-medium transition-all {activeTab === 'replace' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => activeTab = 'replace'}
          >
            Replace
          </button>
        </div>

        <button
          type="button"
          class="text-slate-400 hover:text-white p-1 rounded hover:bg-slate-800 transition-colors"
          onclick={close}
          title="Close (Esc)"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Form Inputs -->
      <div class="p-4 space-y-3.5 text-xs">
        <!-- Find What Input -->
        <div class="flex flex-col gap-1">
          <label for="find-what" class="text-slate-300 font-semibold text-[11px]">Find what:</label>
          <input
            id="find-what"
            type="text"
            bind:value={findText}
            placeholder="Type value or text to find..."
            class="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
            autofocus
          />
        </div>

        <!-- Replace With Input (Only in Replace Tab) -->
        {#if activeTab === 'replace'}
          <div class="flex flex-col gap-1 animate-in fade-in duration-100">
            <label for="replace-with" class="text-slate-300 font-semibold text-[11px]">Replace with:</label>
            <input
              id="replace-with"
              type="text"
              bind:value={replaceText}
              placeholder="Type replacement value..."
              class="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
            />
          </div>
        {/if}

        <!-- Look in Variable Dropdown -->
        <div class="flex flex-col gap-1">
          <label for="target-col" class="text-slate-300 font-semibold text-[11px]">Look in Variable:</label>
          <select
            id="target-col"
            bind:value={targetCol}
            class="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
          >
            <option value="all">All Variables (Entire Dataset)</option>
            {#if appState.activeDataset}
              {#each appState.activeDataset.variables as v}
                <option value={v.name}>{v.name} ({v.label || v.name})</option>
              {/each}
            {/if}
          </select>
        </div>

        <!-- Options Checkboxes -->
        <div class="pt-1 border-t border-slate-800/80 space-y-1.5">
          <label class="flex items-center gap-2 text-slate-300 hover:text-white cursor-pointer select-none">
            <input
              type="checkbox"
              bind:checked={matchEntireCell}
              class="rounded bg-slate-950 border-slate-700 text-indigo-600 focus:ring-0 focus:ring-offset-0"
            />
            <span class="text-[11px]">Match entire cell contents</span>
          </label>

          <label class="flex items-center gap-2 text-slate-300 hover:text-white cursor-pointer select-none">
            <input
              type="checkbox"
              bind:checked={matchCase}
              class="rounded bg-slate-950 border-slate-700 text-indigo-600 focus:ring-0 focus:ring-offset-0"
            />
            <span class="text-[11px]">Match case</span>
          </label>

          <label class="flex items-center gap-2 text-slate-400 hover:text-slate-200 cursor-pointer select-none">
            <input
              type="checkbox"
              bind:checked={useRegex}
              class="rounded bg-slate-950 border-slate-700 text-indigo-600 focus:ring-0 focus:ring-offset-0"
            />
            <span class="text-[11px]">Use regular expressions (RE)</span>
          </label>
        </div>

        <!-- Status Message Banner -->
        {#if statusMessage}
          <div
            class="px-3 py-1.5 rounded text-[11px] flex items-center gap-1.5 {statusType === 'error' ? 'bg-rose-950/60 text-rose-300 border border-rose-800' : statusType === 'success' ? 'bg-emerald-950/60 text-emerald-300 border border-emerald-800' : 'bg-slate-800 text-slate-300'}"
          >
            <span>{statusType === 'error' ? '⚠️' : statusType === 'success' ? '✓' : 'ℹ️'}</span>
            <span>{statusMessage}</span>
          </div>
        {/if}
      </div>

      <!-- Dialog Footer Actions -->
      <div class="px-4 py-2.5 bg-slate-950 border-t border-slate-800 flex items-center justify-between">
        <button
          type="button"
          class="px-3 py-1.5 text-xs text-slate-400 hover:text-white rounded hover:bg-slate-800 transition-colors"
          onclick={close}
        >
          Close
        </button>

        <div class="flex items-center gap-2">
          {#if activeTab === 'replace'}
            <button
              type="button"
              class="px-3 py-1.5 text-xs rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-50"
              onclick={handleReplace}
              disabled={isProcessing || !findText}
            >
              Replace
            </button>
            <button
              type="button"
              class="px-3 py-1.5 text-xs rounded bg-amber-600 hover:bg-amber-500 text-white font-medium shadow-sm transition-colors disabled:opacity-50"
              onclick={handleReplaceAll}
              disabled={isProcessing || !findText}
            >
              Replace All
            </button>
          {/if}

          <button
            type="button"
            class="px-3.5 py-1.5 text-xs rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium shadow-sm transition-colors disabled:opacity-50"
            onclick={findNext}
            disabled={isProcessing || !findText}
          >
            Find Next
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
