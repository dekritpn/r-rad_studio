<!-- src/lib/components/designer/FormDesigner.svelte -->
<!-- VB6 / Qt-Style Visual Form Builder (RAD Mode) -->
<script lang="ts">
  import { appState } from '../../stores/appState.svelte';
  import type { FormWidget, WidgetType } from '../../schemas/rform';
  import type { MeasureType } from '../../schemas/dataset';
  import { compileRTemplate } from '../../runtime/templateEngine';
  import { invoke } from '@tauri-apps/api/core';

  let saveStatus = $state<string | null>(null);
  let livePreviewValues = $state<Record<string, unknown>>({});

  const schema = $derived(appState.designer.schema);
  const selectedWidget = $derived(
    schema.layout.find((w) => w.id === appState.designer.selectedWidgetId) || null
  );

  const availableTokens = $derived(schema.layout.map((w) => w.id));

  // Available toolbox widgets
  const toolboxWidgets: { type: WidgetType; label: string; icon: string; defaultSpan: number }[] = [
    { type: 'VariableSourceList', label: 'Variable Source List', icon: '📋', defaultSpan: 5 },
    { type: 'VariableTargetSlot', label: 'Variable Target Slot', icon: '🎯', defaultSpan: 6 },
    { type: 'Checkbox', label: 'Checkbox', icon: '☑️', defaultSpan: 6 },
    { type: 'NumericInput', label: 'Numeric Input', icon: '🔢', defaultSpan: 6 },
    { type: 'TextInput', label: 'Text Input', icon: '✏️', defaultSpan: 6 },
    { type: 'Dropdown', label: 'Dropdown Select', icon: '▾', defaultSpan: 6 },
    { type: 'RadioGroup', label: 'Radio Group', icon: '🔘', defaultSpan: 6 },
    { type: 'SubDialogButton', label: 'Sub-Dialog Button', icon: '↗️', defaultSpan: 3 },
  ];

  function addWidget(type: WidgetType, label: string, defaultSpan: number) {
    const nextIdx = schema.layout.length + 1;
    const baseId = type.toLowerCase().replace(/[^a-z0-9]/g, '_');
    const newId = `${baseId}_${nextIdx}`;

    const newWidget: FormWidget = {
      id: newId,
      type,
      label: `${label} ${nextIdx}`,
      grid: { x: 1, y: nextIdx, w: defaultSpan, h: 2 },
    };

    if (type === 'VariableTargetSlot') {
      newWidget.accepts = { types: ['scale', 'ordinal', 'nominal'], multiple: false, required: true };
    } else if (type === 'Checkbox') {
      newWidget.default = false;
    } else if (type === 'NumericInput') {
      newWidget.default = 1;
      newWidget.min = 0;
      newWidget.max = 100;
      newWidget.step = 1;
    }

    schema.layout.push(newWidget);
    appState.designer.selectedWidgetId = newId;
  }

  function removeWidget(id: string) {
    schema.layout = schema.layout.filter((w) => w.id !== id);
    if (appState.designer.selectedWidgetId === id) {
      appState.designer.selectedWidgetId = null;
    }
  }

  function moveWidget(index: number, direction: 'up' | 'down') {
    const targetIdx = direction === 'up' ? index - 1 : index + 1;
    if (targetIdx < 0 || targetIdx >= schema.layout.length) return;
    const temp = schema.layout[index];
    schema.layout[index] = schema.layout[targetIdx];
    schema.layout[targetIdx] = temp;
  }

  function insertToken(token: string) {
    schema.r_template += ` {{${token}}}`;
  }

  async function saveDialog() {
    try {
      saveStatus = 'Saving...';
      const formPayload = JSON.parse(JSON.stringify(schema));
      const savedId: string = await invoke('save_custom_form', { formJson: formPayload });

      // Refresh menu tree
      const updatedMenu: any = await invoke('get_forms_menu');
      appState.menuTree = updatedMenu;

      saveStatus = `Saved successfully as "${savedId}"!`;
      setTimeout(() => (saveStatus = null), 3000);
    } catch (err) {
      console.error('Failed to save custom form:', err);
      saveStatus = `Error: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  async function testRunAnalysis() {
    try {
      saveStatus = 'Running test...';
      const compiled = compileRTemplate(schema.r_template, livePreviewValues);
      appState.daemonStatus = 'running';
      const res: any = await invoke('execute_analysis', { rCode: compiled });
      appState.daemonStatus = 'idle';

      if (res.status === 'ok' && res.blocks) {
        appState.appendOutput(`Test: ${schema.window.title}`, res.blocks);
      } else {
        appState.appendOutput(`Test: ${schema.window.title} (Failed)`, [
          { type: 'syntax', content: compiled },
          { type: 'error', content: res.error || 'Execution failed' },
        ]);
      }
      saveStatus = 'Test executed. Check Output tab!';
      setTimeout(() => (saveStatus = null), 3000);
    } catch (err) {
      saveStatus = `Compile Error: ${err instanceof Error ? err.message : String(err)}`;
    }
  }
</script>

<div class="flex flex-col h-full bg-slate-950 text-slate-100 select-none overflow-hidden">
  <!-- Designer Top Header -->
  <div class="h-11 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 z-10">
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-1.5 text-xs font-bold text-amber-400">
        <span>🔧</span>
        <span>RAD Form Designer</span>
      </div>
      <span class="text-slate-600">|</span>
      <input
        type="text"
        bind:value={schema.window.title}
        class="bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs text-white font-semibold focus:outline-none focus:border-indigo-500 w-56"
        placeholder="Dialog Title..."
      />
      <span class="text-[11px] text-slate-400 font-mono">ID: {schema.id}</span>
    </div>

    <!-- Actions & Save -->
    <div class="flex items-center gap-2">
      {#if saveStatus}
        <span class="text-xs text-emerald-400 font-medium animate-pulse">{saveStatus}</span>
      {/if}

      <button
        type="button"
        class="px-3 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded text-xs font-medium transition-colors"
        onclick={testRunAnalysis}
      >
        ▶ Test Run
      </button>

      <button
        type="button"
        class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded text-xs font-bold transition-colors shadow-sm flex items-center gap-1.5"
        onclick={saveDialog}
      >
        <span>💾 Save & Mount to Menu</span>
      </button>
    </div>
  </div>

  <!-- 4-Pane Workspace -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Pane 1: Left Widget Toolbox (VB6/Qt Palette) -->
    <div class="w-56 bg-slate-900 border-r border-slate-800 flex flex-col p-3 overflow-y-auto">
      <div class="text-[10px] font-bold uppercase tracking-wider text-slate-400 mb-2">
        Statistical Toolbox
      </div>
      <div class="space-y-1.5">
        {#each toolboxWidgets as tb}
          <button
            type="button"
            class="w-full text-left px-3 py-2 rounded-lg bg-slate-900 hover:bg-indigo-600 hover:text-white border border-slate-800 hover:border-indigo-500 transition-all text-xs font-medium flex items-center gap-2.5 shadow-sm group"
            onclick={() => addWidget(tb.type, tb.label, tb.defaultSpan)}
          >
            <span class="text-base leading-none group-hover:scale-110 transition-transform">{tb.icon}</span>
            <span class="truncate">{tb.label}</span>
          </button>
        {/each}
      </div>

      <div class="mt-6 pt-4 border-t border-slate-800 text-[11px] text-slate-400">
        <p class="font-semibold text-slate-300 mb-1">Menu Bar Mounting:</p>
        <p class="text-[10px] text-slate-500">
          Saved forms automatically appear in the top menu under:
          <br /><strong class="text-indigo-400">{schema.menu_path.join(' &rarr; ')}</strong>
        </p>
      </div>
    </div>

    <!-- Pane 2: Center Interactive WYSIWYG Grid Canvas -->
    <div class="flex-1 bg-slate-950 p-6 overflow-y-auto flex flex-col items-center">
      <!-- Simulated Window Frame -->
      <div
        class="bg-slate-900 border-2 border-indigo-500/40 rounded-xl shadow-2xl p-4 flex flex-col relative w-full max-w-2xl min-h-[380px]"
      >
        <!-- Mock Window Titlebar -->
        <div class="flex items-center justify-between pb-3 mb-3 border-b border-slate-800">
          <div class="flex items-center gap-2">
            <span class="w-2.5 h-2.5 rounded-full bg-amber-400"></span>
            <span class="text-xs font-bold text-slate-200">{schema.window.title}</span>
          </div>
          <span class="text-[10px] text-slate-500 font-mono">12-Column Grid Canvas</span>
        </div>

        <!-- 12-Column Canvas Grid Area -->
        <div class="grid grid-cols-12 gap-3 flex-1 border border-dashed border-slate-800 rounded-lg p-3 min-h-[260px]">
          {#if schema.layout.length === 0}
            <div class="col-span-12 flex flex-col items-center justify-center text-slate-600 text-xs py-12">
              <span class="text-2xl mb-1">👈</span>
              <span>Click any widget in the Toolbox to place it onto the canvas</span>
            </div>
          {:else}
            {#each schema.layout as w, wIdx}
              {@const isSelected = appState.designer.selectedWidgetId === w.id}
              <div
                class="relative rounded-lg p-2.5 border transition-all cursor-pointer flex flex-col justify-between {isSelected ? 'border-indigo-500 bg-indigo-950/20 shadow-lg ring-1 ring-indigo-500' : 'border-slate-800 bg-slate-900 hover:border-slate-700'}"
                style="grid-column: span {w.grid?.w ?? 6} / span {w.grid?.w ?? 6};"
                onclick={() => (appState.designer.selectedWidgetId = w.id)}
              >
                <!-- Mini Header badge -->
                <div class="flex items-center justify-between mb-1.5">
                  <div class="flex items-center gap-1.5">
                    <span class="text-xs font-mono font-bold text-indigo-400">{w.id}</span>
                    <span class="text-[9px] bg-slate-800 px-1.5 py-0.5 rounded text-slate-400">{w.type}</span>
                  </div>
                  <!-- Move / Delete Actions -->
                  <div class="flex items-center gap-1">
                    <button
                      class="text-slate-500 hover:text-slate-300 text-xs px-1"
                      onclick={(e) => { e.stopPropagation(); moveWidget(wIdx, 'up'); }}
                      title="Move up"
                    >
                      &uarr;
                    </button>
                    <button
                      class="text-slate-500 hover:text-slate-300 text-xs px-1"
                      onclick={(e) => { e.stopPropagation(); moveWidget(wIdx, 'down'); }}
                      title="Move down"
                    >
                      &darr;
                    </button>
                    <button
                      class="text-slate-500 hover:text-rose-400 text-xs px-1 font-bold"
                      onclick={(e) => { e.stopPropagation(); removeWidget(w.id); }}
                      title="Delete widget"
                    >
                      &times;
                    </button>
                  </div>
                </div>

                <!-- Preview Representation -->
                <div class="text-xs text-slate-300 font-medium">
                  {w.label || w.id}
                </div>
                <div class="text-[10px] text-slate-500 mt-1 font-mono">
                  Span: {w.grid?.w ?? 6} cols
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>

    <!-- Pane 3: Right Property Inspector -->
    <div class="w-72 bg-slate-900 border-l border-slate-800 p-4 flex flex-col overflow-y-auto">
      <div class="text-[10px] font-bold uppercase tracking-wider text-slate-400 mb-3">
        Property Inspector
      </div>

      {#if !selectedWidget}
        <div class="text-xs text-slate-500 italic p-3 text-center">
          Select a widget on the canvas to inspect and edit its properties
        </div>
      {:else}
        <div class="space-y-3 text-xs">
          <div>
            <label class="text-[11px] font-semibold text-slate-300">Widget Identifier (ID)</label>
            <input
              type="text"
              bind:value={selectedWidget.id}
              class="w-full mt-1 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-indigo-300 focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label class="text-[11px] font-semibold text-slate-300">Display Label</label>
            <input
              type="text"
              bind:value={selectedWidget.label}
              class="w-full mt-1 bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label class="text-[11px] font-semibold text-slate-300">Column Span (1 to 12)</label>
            <div class="flex items-center gap-2 mt-1">
              <input
                type="range"
                min="1"
                max="12"
                step="1"
                value={selectedWidget.grid?.w ?? 6}
                oninput={(e) => {
                  if (selectedWidget.grid) {
                    selectedWidget.grid.w = parseInt((e.target as HTMLInputElement).value);
                  }
                }}
                class="flex-1 accent-indigo-500"
              />
              <span class="font-mono text-xs text-indigo-300 w-6 text-right">
                {selectedWidget.grid?.w ?? 6}
              </span>
            </div>
          </div>

          <!-- Type-Specific Inspector Properties -->
          {#if selectedWidget.type === 'VariableTargetSlot'}
            <div class="pt-2 border-t border-slate-800 space-y-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={selectedWidget.accepts?.multiple}
                  onchange={(e) => {
                    if (!selectedWidget.accepts) selectedWidget.accepts = {};
                    selectedWidget.accepts.multiple = (e.target as HTMLInputElement).checked;
                  }}
                  class="rounded bg-slate-950 border-slate-700 text-indigo-500"
                />
                <span>Allow Multiple Variables</span>
              </label>

              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={selectedWidget.accepts?.required}
                  onchange={(e) => {
                    if (!selectedWidget.accepts) selectedWidget.accepts = {};
                    selectedWidget.accepts.required = (e.target as HTMLInputElement).checked;
                  }}
                  class="rounded bg-slate-950 border-slate-700 text-indigo-500"
                />
                <span>Required Field</span>
              </label>
            </div>

          {:else if selectedWidget.type === 'NumericInput'}
            <div class="pt-2 border-t border-slate-800 grid grid-cols-2 gap-2">
              <div>
                <label class="text-[10px] text-slate-400">Min</label>
                <input
                  type="number"
                  bind:value={selectedWidget.min}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-white"
                />
              </div>
              <div>
                <label class="text-[10px] text-slate-400">Max</label>
                <input
                  type="number"
                  bind:value={selectedWidget.max}
                  class="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-xs text-white"
                />
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Pane 4: Bottom R Code Template Editor with Token Autocomplete -->
  <div class="h-44 bg-slate-900 border-t border-slate-800 flex flex-col p-3 z-10">
    <div class="flex items-center justify-between mb-1.5">
      <div class="flex items-center gap-2">
        <span class="text-blue-400 font-mono font-bold text-xs">R</span>
        <span class="text-xs font-semibold text-slate-200">R Code Template Editor</span>
        <span class="text-[10px] text-slate-500 italic">Interpolates bound tokens using Handlebars</span>
      </div>

      <!-- Quick Token Insert Badges -->
      <div class="flex items-center gap-1">
        <span class="text-[10px] text-slate-400 mr-1">Insert Token:</span>
        {#each availableTokens as tok}
          <button
            type="button"
            class="px-2 py-0.5 rounded bg-slate-800 hover:bg-indigo-600 text-indigo-300 hover:text-white text-[10px] font-mono border border-slate-700 transition-colors"
            onclick={() => insertToken(tok)}
          >
            {`{{${tok}}}`}
          </button>
        {/each}
      </div>
    </div>

    <textarea
      bind:value={schema.r_template}
      class="flex-1 bg-slate-950 border border-slate-800 rounded-lg p-2.5 font-mono text-xs text-emerald-300 focus:outline-none focus:border-indigo-500 resize-none"
      placeholder="local( ... )"
    ></textarea>
  </div>
</div>
