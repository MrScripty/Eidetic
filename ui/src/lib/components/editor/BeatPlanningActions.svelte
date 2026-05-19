<script lang="ts">
  let {
    childLevelName,
    hasChildren,
    planning,
    notes,
    onplan,
  }: {
    childLevelName: string;
    hasChildren: boolean;
    planning: boolean;
    notes: string;
    onplan: () => void;
  } = $props();
</script>

<div class="beat-actions">
  <button class="plan-btn" onclick={onplan} disabled={planning || !notes.trim()}>
    {planning
      ? 'Planning...'
      : hasChildren
        ? `Replan ${childLevelName}s`
        : `Plan ${childLevelName}s`}
  </button>
</div>
