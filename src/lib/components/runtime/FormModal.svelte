<!-- src/lib/components/runtime/FormModal.svelte -->
<!-- Dynamic Statistical Form Renderer (SPSS Emulation Engine) -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import type { FormWidget, RFormSchema, SubDialogDefinition } from '../../schemas/rform';
  import VariableSourceList from './widgets/VariableSourceList.svelte';
  import VariableTargetSlot from './widgets/VariableTargetSlot.svelte';
  import StandardWidgets from './widgets/StandardWidgets.svelte';
  import { compileRTemplate } from '../../runtime/templateEngine';
  import { invoke } from '@tauri-apps/api/core';

  let selectedSourceVar = $state<string | null>(null);
  let validationMessage = $state<string | null>(null);

  const schema = $derived(appState.activeDialog.schema);
  const activeSubDialog = $derived(
    schema?.sub_dialogs?.find((s) => s.id === appState.activeDialog.subDialogId) || null
  );

  // Computes which variables are already placed in target slots
  const allocatedVariables = $derived.by(() => {
    const list: string[] = [];
    if (!schema?.layout) return list;
    for (const w of schema.layout) {
      if (w.type === 'VariableTargetSlot') {
        const val = appState.activeDialog.values[w.id];
        if (Array.isArray(val)) {
          list.push(...val);
        } else if (typeof val === 'string' && val) {
          list.push(val);
        }
      }
    }
    return list;
  });

  // Check if form is valid
  function validateForm(): boolean {
    if (!schema?.layout) return false;
    for (const w of schema.layout) {
      if (w.type === 'VariableTargetSlot' && w.accepts?.required) {
        const val = appState.activeDialog.values[w.id];
        if (!val || (Array.isArray(val) && val.length === 0)) {
          validationMessage = `Please fill required slot: ${w.label || w.id}`;
          setTimeout(() => (validationMessage = null), 3000);
          return false;
        }
      }
    }
    return true;
  }

  // Build template context
  function buildTemplateContext(): Record<string, unknown> {
    const ctx: Record<string, unknown> = { ...appState.activeDialog.values };
    // Add sub_dialogs dictionary
    const subDict: Record<string, any> = {};
    if (schema?.sub_dialogs) {
      for (const sub of schema.sub_dialogs) {
        subDict[sub.id] = {};
        for (const w of sub.layout) {
          const fullKey = `${sub.id}_${w.id}`;
          subDict[sub.id][w.id] = appState.activeDialog.values[fullKey] ?? w.default;
        }
      }
    }
    ctx.sub_dialogs = subDict;
    return ctx;
  }

  // [ OK ] Action
  async function handleOk() {
    if (!validateForm() || !schema) return;

    const dialogTitle = schema.window?.title || 'Analysis Output';
    const template = schema.r_template;

    try {
      const context = buildTemplateContext();
      const compiledR = compileRTemplate(template, context);

      appState.daemonStatus = 'running';
      appState.closeDialog();

      const result: any = await invoke('execute_analysis', { rCode: compiledR });
      appState.daemonStatus = 'idle';

      if (result.status === 'ok' && result.blocks) {
        appState.appendOutput(dialogTitle, result.blocks);
      } else {
        appState.appendOutput(`${dialogTitle} (Failed)`, [
          { type: 'syntax', content: compiledR },
          { type: 'error', content: result.error || 'Unknown analysis error' },
        ]);
      }
    } catch (err) {
      console.error('Error running analysis:', err);
      appState.daemonStatus = 'error';
      appState.appendOutput(`${dialogTitle} (Error)`, [
        { type: 'error', content: err instanceof Error ? err.message : String(err) },
      ]);
    }
  }

  // [ Paste ] Action
  function handlePaste() {
    if (!validateForm() || !schema) return;
    const dialogTitle = schema.window?.title || 'Analysis';
    try {
      const context = buildTemplateContext();
      const compiledR = compileRTemplate(schema.r_template, context);

      appState.syntaxCode += `\n# --- ${dialogTitle} ---\n${compiledR}\n`;
      appState.closeDialog();
      appState.setView('output');
    } catch (err) {
      validationMessage = err instanceof Error ? err.message : String(err);
    }
  }

  // [ Reset ] Action
  function handleReset() {
    if (!schema?.layout) return;
    const freshValues: Record<string, unknown> = {};
    for (const w of schema.layout) {
      if (w.default !== undefined) {
        freshValues[w.id] = w.default;
      } else if (w.type === 'VariableTargetSlot') {
        freshValues[w.id] = w.accepts?.multiple ? [] : null;
      }
    }
    appState.activeDialog.values = freshValues;
    selectedSourceVar = null;
  }

  // Customize in Form Designer (RAD mode)
  function handleCustomize() {
    if (schema) {
      appState.openDesignerWith(schema);
      appState.closeDialog();
    }
  }
</script>

{#if appState.activeDialog.isOpen && schema}
  <!-- Backdrop -->
  <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <!-- Main Dialog Window -->
    <div
      class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl flex flex-col overflow-hidden max-w-full max-h-[90vh]"
      style="width: {schema?.window?.width ?? 640}px; height: {schema?.window?.height ?? 460}px;"
    >
      <!-- Title Bar -->
      <div class="h-10 bg-slate-900 border-b border-slate-800 px-4 flex items-center justify-between select-none">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-indigo-500"></span>
          <h2 class="text-xs font-bold text-white tracking-wide">{schema?.window?.title ?? 'Analysis Dialog'}</h2>
        </div>

        <div class="flex items-center gap-2">
          <!-- Customize in Form Designer Button -->
          <button
            type="button"
            class="px-2 py-0.5 rounded bg-slate-800 hover:bg-amber-600 hover:text-white text-amber-300 text-[11px] font-medium transition-colors flex items-center gap-1 border border-slate-700"
            onclick={handleCustomize}
            title="Open in Visual Form Designer to edit widgets or R template"
          >
            <span>🔧 Customize Dialog</span>
          </button>

          <!-- Close Window -->
          <button
            type="button"
            class="text-slate-400 hover:text-white text-base px-1.5 py-0.5 font-bold transition-colors"
            onclick={() => appState.closeDialog()}
          >
            &times;
          </button>
        </div>
      </div>

      <!-- Main 12-Column Grid Canvas Body -->
      <div class="flex-1 p-4 overflow-y-auto relative">
        <div class="grid grid-cols-12 gap-3 h-full">
          {#each schema?.layout || [] as widget}
            {@const gx = widget.grid?.x ?? 1}
            {@const gw = widget.grid?.w ?? 12}
            <div
              class="flex flex-col"
              style="grid-column: span {gw} / span {gw};"
            >
              {#if widget.type === 'VariableSourceList'}
                <VariableSourceList
                  id={widget.id}
                  filter_types={widget.filter_types}
                  {selectedSourceVar}
                  onSelectVariable={(v) => (selectedSourceVar = v)}
                  onDoubleClickVariable={(v) => {
                    // Auto-transfer to first empty or accept slot
                    selectedSourceVar = v;
                    const firstSlot = schema?.layout?.find((w) => w.type === 'VariableTargetSlot');
                    if (firstSlot) {
                      const cur = appState.activeDialog.values[firstSlot.id];
                      if (firstSlot.accepts?.multiple) {
                        const arr = Array.isArray(cur) ? [...cur] : [];
                        if (!arr.includes(v)) {
                          arr.push(v);
                          appState.activeDialog.values[firstSlot.id] = arr;
                        }
                      } else {
                        appState.activeDialog.values[firstSlot.id] = v;
                      }
                    }
                  }}
                  {allocatedVariables}
                />

              {:else if widget.type === 'VariableTargetSlot'}
                <VariableTargetSlot
                  id={widget.id}
                  label={widget.label}
                  accepts={widget.accepts}
                  value={appState.activeDialog.values[widget.id] as string | string[] | null}
                  {selectedSourceVar}
                  onChange={(val) => (appState.activeDialog.values[widget.id] = val)}
                />

              {:else}
                <StandardWidgets
                  {widget}
                  value={appState.activeDialog.values[widget.id]}
                  onChange={(val) => (appState.activeDialog.values[widget.id] = val)}
                  onOpenSubDialog={(subId) => (appState.activeDialog.subDialogId = subId)}
                />
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Error Toast -->
      {#if validationMessage}
        <div class="px-4 py-1.5 bg-rose-950/80 border-t border-rose-800 text-rose-300 text-xs flex items-center gap-2">
          <span>⚠️</span>
          <span>{validationMessage}</span>
        </div>
      {/if}

      <!-- Bottom Action Bar (Classic SPSS Actions) -->
      <div class="h-12 bg-slate-950 border-t border-slate-800 px-4 flex items-center justify-between select-none">
        <div class="flex items-center gap-1.5">
          <button
            type="button"
            class="px-3 py-1 bg-slate-800 hover:bg-slate-750 text-slate-300 rounded text-xs transition-colors"
            onclick={handleReset}
          >
            Reset
          </button>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            class="px-4 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-xs font-semibold shadow transition-colors flex items-center gap-1.5"
            onclick={handleOk}
          >
            <span>OK</span>
          </button>

          <button
            type="button"
            class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded text-xs font-medium transition-colors"
            onclick={handlePaste}
            title="Paste R syntax to editor tab without executing"
          >
            Paste
          </button>

          <button
            type="button"
            class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-xs font-medium transition-colors"
            onclick={() => appState.closeDialog()}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Sub-Dialog Modal (Child Modal for Options/Statistics) -->
  {#if activeSubDialog}
    <div class="fixed inset-0 bg-black/60 z-[60] flex items-center justify-center p-4">
      <div class="bg-slate-900 border border-slate-700 rounded-xl shadow-2xl w-96 overflow-hidden flex flex-col">
        <div class="h-9 bg-slate-900 px-3 flex items-center justify-between border-b border-slate-800">
          <span class="text-xs font-bold text-white">{activeSubDialog.label}</span>
          <button
            class="text-slate-400 hover:text-white text-base font-bold"
            onclick={() => (appState.activeDialog.subDialogId = null)}
          >
            &times;
          </button>
        </div>

        <div class="p-4 space-y-3">
          {#each activeSubDialog.layout as subWidget}
            {@const fullKey = `${activeSubDialog.id}_${subWidget.id}`}
            <StandardWidgets
              widget={subWidget}
              value={appState.activeDialog.values[fullKey] ?? subWidget.default}
              onChange={(val) => (appState.activeDialog.values[fullKey] = val)}
            />
          {/each}
        </div>

        <div class="h-10 bg-slate-950 border-t border-slate-800 px-3 flex items-center justify-end gap-2">
          <button
            type="button"
            class="px-3 py-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-xs font-semibold"
            onclick={() => (appState.activeDialog.subDialogId = null)}
          >
            Continue
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}
