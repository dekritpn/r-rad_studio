<!-- src/lib/components/runtime/widgets/VariableTargetSlot.svelte -->
<!-- Target slot receiving variables with type and cardinality validation -->
<script lang="ts">
  import { appState } from '../../../stores/appState.svelte';
  import type { TargetSlotAccepts } from '../../../schemas/rform';

  interface Props {
    id: string;
    label?: string;
    accepts?: TargetSlotAccepts;
    value: string | string[] | null;
    selectedSourceVar: string | null;
    onChange: (val: string | string[] | null) => void;
  }

  let {
    id,
    label = 'Variables:',
    accepts = { types: ['scale', 'ordinal', 'nominal'], multiple: false, required: false },
    value,
    selectedSourceVar,
    onChange,
  }: Props = $props();

  let validationError = $state<string | null>(null);

  const currentList = $derived(
    Array.isArray(value) ? value : value ? [value] : []
  );

  function assignSelectedVariable() {
    if (!selectedSourceVar || !appState.activeDataset) return;
    const varMeta = appState.activeDataset.variables.find((v) => v.name === selectedSourceVar);
    if (!varMeta) return;

    // Validate type constraints
    if (accepts.types && accepts.types.length > 0 && !accepts.types.includes(varMeta.measure)) {
      validationError = `Accepts only [${accepts.types.join(', ')}], but '${varMeta.name}' is ${varMeta.measure}`;
      setTimeout(() => (validationError = null), 3000);
      return;
    }

    if (accepts.multiple) {
      const arr = Array.isArray(value) ? [...value] : [];
      if (!arr.includes(selectedSourceVar)) {
        arr.push(selectedSourceVar);
        onChange(arr);
      }
    } else {
      onChange(selectedSourceVar);
    }
  }

  function removeVariable(varName: string) {
    if (accepts.multiple) {
      const arr = Array.isArray(value) ? value.filter((v) => v !== varName) : [];
      onChange(arr);
    } else {
      onChange(null);
    }
  }
</script>

<div class="flex flex-col h-full select-none">
  <!-- Slot Label -->
  <div class="flex items-center justify-between mb-1">
    <label class="text-xs font-semibold text-slate-200 flex items-center gap-1">
      {label}
      {#if accepts.required}
        <span class="text-rose-400 font-bold">*</span>
      {/if}
    </label>
    {#if accepts.types && accepts.types.length > 0}
      <span class="text-[10px] text-slate-400 font-mono">[{accepts.types.join('/')}]</span>
    {/if}
  </div>

  <!-- Main Slot Box + Arrow Transfer Button -->
  <div class="flex items-start gap-1.5 flex-1 min-h-[50px]">
    <!-- Directional Arrow Button [ > ] -->
    <button
      type="button"
      class="mt-1 px-2 py-1 bg-slate-800 hover:bg-indigo-600 disabled:opacity-40 disabled:hover:bg-slate-800 text-slate-200 rounded text-xs font-bold transition-colors shadow-sm"
      disabled={!selectedSourceVar}
      onclick={assignSelectedVariable}
      title="Transfer selected variable into this slot"
    >
      &rarr;
    </button>

    <!-- Drop/List Container -->
    <div class="flex-1 h-full min-h-[60px] bg-slate-900 border {validationError ? 'border-rose-500' : currentList.length === 0 && accepts.required ? 'border-amber-600/60' : 'border-slate-700'} rounded-lg p-1.5 overflow-y-auto">
      {#if currentList.length === 0}
        <div class="h-full flex items-center justify-center text-xs text-slate-500 italic">
          {accepts.multiple ? 'No variables assigned' : 'No variable assigned'}
        </div>
      {:else}
        <div class="space-y-1">
          {#each currentList as item}
            {@const vMeta = appState.activeDataset?.variables.find((v) => v.name === item)}
            <div class="flex items-center justify-between px-2 py-1 bg-slate-800 text-slate-100 rounded text-xs border border-slate-700/80">
              <div class="flex items-center gap-1.5 truncate">
                {#if vMeta?.measure === 'scale'}
                  <span class="text-amber-400 font-bold text-xs">📐</span>
                {:else if vMeta?.measure === 'ordinal'}
                  <span class="text-sky-400 font-bold text-xs">📊</span>
                {:else}
                  <span class="text-emerald-400 font-bold text-xs">⚪</span>
                {/if}
                <span class="font-mono font-medium truncate">{item}</span>
              </div>
              <button
                type="button"
                class="text-slate-400 hover:text-rose-400 text-xs px-1 font-bold transition-colors"
                onclick={() => removeVariable(item)}
                title="Remove variable"
              >
                &times;
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Error feedback message -->
  {#if validationError}
    <div class="text-[10px] text-rose-400 mt-1 font-medium">{validationError}</div>
  {/if}
</div>
