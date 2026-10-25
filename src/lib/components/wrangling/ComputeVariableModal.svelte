<!-- src/lib/components/wrangling/ComputeVariableModal.svelte -->
<!-- SPSS Emulation: Transform -> Compute Variable Dialog -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let targetVar = $state<string>('');
  let expression = $state<string>('');
  let selectedSourceVar = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let isExecuting = $state<boolean>(false);

  const variables = $derived(appState.activeDataset?.variables || []);

  function insertToken(token: string) {
    expression = expression + token;
  }

  function insertVariable(varName: string) {
    // If varName contains spaces or special chars, escape with backticks
    const escaped = /[^a-zA-Z0-9._]/.test(varName) ? `\`${varName}\`` : varName;
    expression = expression + (expression && !expression.endsWith(' ') ? ' ' : '') + escaped;
  }

  async function handleOk() {
    errorMessage = null;
    const cleanTarget = targetVar.trim();
    const cleanExpr = expression.trim();

    if (!cleanTarget) {
      errorMessage = 'Please specify a target variable name.';
      return;
    }
    if (!cleanExpr) {
      errorMessage = 'Please specify a numeric formula expression.';
      return;
    }

    try {
      isExecuting = true;
      appState.daemonStatus = 'running';

      const res: any = await invoke('compute_variable', {
        targetVar: cleanTarget,
        expression: cleanExpr,
      });

      appState.daemonStatus = 'idle';

      // Update dataset metadata
      appState.setDataset(res.dataset);

      // Refresh data page slice
      const slice: any = await invoke('get_data_page', {
        offset: appState.dataOffset,
        limit: appState.dataLimit,
      });
      appState.setDataPage(slice.rows, slice.total_rows, slice.offset);

      // Emit output block
      if (res.blocks && res.blocks.length > 0) {
        appState.appendOutput(`Compute Variable: ${cleanTarget}`, res.blocks);
      }

      appState.showToast(`Computed variable '${cleanTarget}' successfully!`);
      appState.isComputeDialogOpen = false;
      targetVar = '';
      expression = '';
    } catch (err) {
      console.error('Compute Variable failed:', err);
      appState.daemonStatus = 'error';
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isExecuting = false;
    }
  }

  function handlePaste() {
    const cleanTarget = targetVar.trim() || 'NEW_VAR';
    const cleanExpr = expression.trim() || 'NA';
    const rCode = `\n# --- Compute Variable ---\n.active_data[['${cleanTarget}']] <- with(.active_data, ${cleanExpr})\n`;
    appState.syntaxCode += rCode;
    appState.showToast('Pasted Compute Variable syntax to Syntax Editor');
    appState.isComputeDialogOpen = false;
    appState.setView('output');
  }

  function handleCancel() {
    errorMessage = null;
    appState.isComputeDialogOpen = false;
  }
</script>

{#if appState.isComputeDialogOpen}
  <!-- Backdrop -->
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <!-- Main Modal Window -->
    <div class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col overflow-hidden w-[680px] max-w-full max-h-[90vh]">
      <!-- Title Bar -->
      <div class="h-10 bg-slate-900 border-b border-slate-800 px-4 flex items-center justify-between select-none">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-indigo-500"></span>
          <h2 class="text-xs font-bold text-white tracking-wide">Compute Variable</h2>
        </div>
        <button
          type="button"
          class="text-slate-400 hover:text-white text-base px-1.5 py-0.5 font-bold transition-colors"
          onclick={handleCancel}
        >
          &times;
        </button>
      </div>

      <!-- Main Body -->
      <div class="p-4 space-y-4 overflow-y-auto">
        <!-- Target Variable Input -->
        <div class="flex items-center gap-3">
          <label class="text-xs font-semibold text-slate-300 w-28 shrink-0">Target Variable:</label>
          <input
            type="text"
            bind:value={targetVar}
            placeholder="e.g. log_mpg, bmi, score_total"
            class="flex-1 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
          />
        </div>

        <!-- 2-Column Split: Variables Palette (Left) & Formula + Keypad (Right) -->
        <div class="grid grid-cols-12 gap-3 min-h-[260px]">
          <!-- Left: Variable List -->
          <div class="col-span-5 bg-slate-950 border border-slate-800 rounded-lg p-2.5 flex flex-col">
            <span class="text-[11px] font-semibold text-slate-400 mb-2">Variables:</span>
            <div class="flex-1 overflow-y-auto space-y-1 max-h-56">
              {#each variables as v}
                <div
                  class="px-2 py-1 rounded text-xs flex items-center justify-between cursor-pointer transition-colors {selectedSourceVar === v.name ? 'bg-indigo-600 text-white font-semibold' : 'text-slate-200 hover:bg-slate-800'}"
                  onclick={() => (selectedSourceVar = v.name)}
                  ondblclick={() => insertVariable(v.name)}
                >
                  <div class="flex items-center gap-1.5 truncate">
                    {#if v.measure === 'scale'}
                      <span class="text-amber-400 font-bold shrink-0 text-xs">📐</span>
                    {:else if v.measure === 'ordinal'}
                      <span class="text-sky-400 font-bold shrink-0 text-xs">📊</span>
                    {:else}
                      <span class="text-emerald-400 font-bold shrink-0 text-xs">⚪</span>
                    {/if}
                    <span class="font-mono truncate">{v.name}</span>
                  </div>
                  <button
                    type="button"
                    class="text-[10px] text-slate-400 hover:text-white px-1 font-mono"
                    onclick={(e) => { e.stopPropagation(); insertVariable(v.name); }}
                    title="Insert into formula"
                  >
                    &rarr;
                  </button>
                </div>
              {/each}
            </div>
          </div>

          <!-- Right: Formula Area & Calculator Keypad -->
          <div class="col-span-7 flex flex-col gap-2">
            <span class="text-[11px] font-semibold text-slate-400">Numeric Expression =</span>
            <textarea
              bind:value={expression}
              rows="3"
              placeholder="e.g. log(mpg) * 2 or weight / (height / 100)^2"
              class="w-full bg-slate-950 border border-slate-700 rounded p-2 text-xs text-emerald-400 font-mono focus:outline-none focus:border-indigo-500 resize-none"
            ></textarea>

            <!-- Keypad Matrix -->
            <div class="bg-slate-950 border border-slate-800 rounded-lg p-2 space-y-2">
              <!-- Operators Row -->
              <div class="flex flex-wrap gap-1">
                {#each ['+', '-', '*', '/', '^', '(', ')', '<', '>', '<=', '>=', '==', '!=', '&', '|'] as op}
                  <button
                    type="button"
                    class="px-2 py-1 bg-slate-800 hover:bg-indigo-600 hover:text-white text-slate-300 font-mono text-xs rounded transition-colors"
                    onclick={() => insertToken(` ${op} `)}
                  >
                    {op}
                  </button>
                {/each}
              </div>

              <!-- Statistical Functions Row -->
              <div class="flex flex-wrap gap-1 border-t border-slate-800 pt-1.5">
                {#each [
                  { label: 'log()', code: 'log()' },
                  { label: 'exp()', code: 'exp()' },
                  { label: 'sqrt()', code: 'sqrt()' },
                  { label: 'abs()', code: 'abs()' },
                  { label: 'round()', code: 'round(, 2)' },
                  { label: 'ifelse()', code: 'ifelse(test, yes, no)' },
                  { label: 'mean()', code: 'mean(, na.rm = TRUE)' },
                  { label: 'sum()', code: 'sum(, na.rm = TRUE)' },
                  { label: 'sd()', code: 'sd(, na.rm = TRUE)' }
                ] as fn}
                  <button
                    type="button"
                    class="px-1.5 py-0.5 bg-slate-850 hover:bg-indigo-700 hover:text-white text-indigo-300 font-mono text-[11px] rounded border border-slate-800 transition-colors"
                    onclick={() => insertToken(fn.code)}
                  >
                    {fn.label}
                  </button>
                {/each}
              </div>
            </div>
          </div>
        </div>

        <!-- Error Message Banner -->
        {#if errorMessage}
          <div class="px-3 py-2 bg-rose-950/80 border border-rose-800 text-rose-300 rounded text-xs flex items-center gap-2">
            <span>⚠️</span>
            <span>{errorMessage}</span>
          </div>
        {/if}
      </div>

      <!-- Bottom Action Bar -->
      <div class="h-12 bg-slate-950 border-t border-slate-800 px-4 flex items-center justify-end gap-2 select-none">
        <button
          type="button"
          class="px-4 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded text-xs font-semibold shadow transition-colors flex items-center gap-1.5"
          onclick={handleOk}
          disabled={isExecuting}
        >
          <span>{isExecuting ? 'Computing...' : 'OK'}</span>
        </button>

        <button
          type="button"
          class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded text-xs font-medium transition-colors"
          onclick={handlePaste}
        >
          Paste
        </button>

        <button
          type="button"
          class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-xs font-medium transition-colors"
          onclick={handleCancel}
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
