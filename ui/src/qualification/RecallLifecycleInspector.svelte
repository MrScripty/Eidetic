<script lang="ts">
  import { onMount } from 'svelte';
  import BibleRecall from '../lib/components/sidebar/bible/BibleRecall.svelte';
  import { lifecycleQA } from './recallLifecycle.svelte.js';
  let { nodeId }: { nodeId: string } = $props();
  onMount(() => {
    lifecycleQA.hosts += 1;
    return () => {
      lifecycleQA.hosts -= 1;
    };
  });
</script>

{#if lifecycleQA.inspectorsVisible}<BibleRecall {nodeId} />{/if}
