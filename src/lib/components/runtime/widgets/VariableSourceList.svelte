<!-- src/lib/components/runtime/widgets/VariableSourceList.svelte -->
<!-- Source list displaying active dataset variables with measurement icons -->
<script lang="ts">
  import { appState } from '../../../stores/appState.svelte';
  import type { MeasureType } from '../../../schemas/dataset';

  interface Props {
    id: string;
    filter_types?: MeasureType[];
    selectedSourceVar: string | null;
    onSelectVariable: (name: string) => void;
    onDoubleClickVariable?: (name: string) => void;
    allocatedVariables?: string[];
  }

  let {
    id,
    filter_types = ['scale', 'ordinal', 'nominal'],
    selectedSourceVar,
    onSelectVariable,
    onDoubleClickVariable,
    allocatedVariables = [],
  }: Props = $props();

  let searchQuery = $state('');

  const availableVariables = $derived(
    (appState.activeDataset?.variables || []).filter((v) => {
      // Filter out allocated variables
      if (allocatedVariables.includes(v.name)) return false;
      // Filter by measure type
      if (filter_types && filter_types.length > 0 && !filter_types.includes(v.measure)) {
        return false;
      }
      // Search query
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        return v.name.toLowerCase().includes(q) || v.label.toLowerCase().includes(q);
      }
      return true;
    })
  );
</script>

<div class="flex flex-col h-full bg-slate-900 border border-slate-700 rounded-lg overflow-hidden select-none">
  <!-- Search Header -->
  <div class="p-2 border-b border-slate-700 bg-slate-900">
    <input
      type="text"
      placeholder="Filter variables..."
      bind:value={searchQuery}
      class="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-indigo-500"
    />
  </div>

  <!-- Variable List Container -->
  <div class="flex-1 overflow-y-auto p-1 space-y-0.5">
    {#if availableVariables.length === 0}
      <div class="p-3 text-center text-xs text-slate-500 italic">
        {searchQuery ? 'No matching variables' : 'No available variables'}
      </div>
    {:else}
      {#each availableVariables as v}
        <div
          class="px-2 py-1.5 rounded text-xs flex items-center justify-between cursor-pointer transition-colors {selectedSourceVar === v.name ? 'bg-indigo-600 text-white font-medium' : 'text-slate-200 hover:bg-slate-800'}"
          onclick={() => onSelectVariable(v.name)}
          ondblclick={() => onDoubleClickVariable && onDoubleClickVariable(v.name)}
        >
          <div class="flex items-center gap-1.5 truncate">
            {#if v.measure === 'scale'}
              <span class="text-amber-400 font-bold shrink-0 text-xs" title="Scale">📐</span>
            {:else if v.measure === 'ordinal'}
              <span class="text-sky-400 font-bold shrink-0 text-xs" title="Ordinal">📊</span>
            {:else}
              <span class="text-emerald-400 font-bold shrink-0 text-xs" title="Nominal">⚪</span>
            {/if}
            <span class="font-mono truncate">{v.name}</span>
          </div>
          {#if v.label}
            <span class="text-[10px] text-slate-400 truncate max-w-[90px] ml-1">{v.label}</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
