<script lang="ts">
  import type { AiBibleContextField, ChildPlanBibleContext } from '$lib/childPlanningTypes.js';
  import type { BibleGraphEdgeKind } from '$lib/bibleGraphTypes.js';
  import type { FieldValue } from '$lib/projectionTypes.js';

  let { evidence }: { evidence?: ChildPlanBibleContext | null } = $props();

  function fieldValueText(value: FieldValue): string {
    switch (value.type) {
      case 'text':
      case 'asset_ref':
        return value.value;
      case 'integer':
      case 'number':
      case 'bool':
        return String(value.value);
      case 'object_ref':
        return `${value.value.kind}: ${value.value.id}`;
    }
  }

  function fieldInput(nodeId: string, field: AiBibleContextField) {
    return evidence?.inputs.find(
      (input) =>
        input.node_id === nodeId &&
        input.part_key === field.part_key &&
        input.field_key === field.field_key,
    );
  }

  function nodeName(nodeId: string): string {
    return evidence?.context.payload.nodes.find((node) => node.node_id === nodeId)?.name ?? nodeId;
  }

  function edgeKindText(kind: BibleGraphEdgeKind): string {
    return typeof kind === 'string' ? kind.replaceAll('_', ' ') : kind.custom;
  }
</script>

<details>
  <summary>Bible facts used for this plan</summary>
  <section aria-label="Recorded Bible evidence">
    {#if evidence == null}
      <p>Bible evidence was not recorded for this plan.</p>
    {:else}
      <p>Recorded when this plan was generated. Later Bible edits do not change this evidence.</p>
      {#if evidence.context.payload.story_time_ms == null}
        <p>Fictional story time was unspecified. Timed fields were withheld.</p>
      {:else}
        <p>Fictional story time: {evidence.context.payload.story_time_ms}ms.</p>
      {/if}
      {#if evidence.context.payload.nodes.length === 0}
        <p>No Bible nodes were supplied to this plan.</p>
      {:else}
        {#each evidence.context.payload.nodes as node (node.node_id)}
          <article data-bible-node={node.node_id}>
            <h4>{node.name} [{node.schema_key}]</h4>
            {#if node.parent_id}<p>Parent: {nodeName(node.parent_id)}</p>{/if}
            {#each node.fields as field}
              {@const input = fieldInput(node.node_id, field)}
              <div
                data-bible-field={input?.field_id}
                data-source-revision={input?.revision_event_id}
              >
                <p>{field.part_key}.{field.field_key}</p>
                <pre>{fieldValueText(field.value)}</pre>
                {#if input}
                  <p>Field revision: {input.revision_event_id}</p>
                {:else}
                  <p>Field revision was not recorded.</p>
                {/if}
              </div>
            {/each}
            {#each node.snapshots as snapshot}
              <h5>Effective fact from {snapshot.label} at fictional time {snapshot.at_ms}ms</h5>
              {#each snapshot.fields as field}
                <p>{field.part_key}.{field.field_key}</p>
                <pre>{fieldValueText(field.value)}</pre>
              {/each}
            {/each}
            {#each node.unresolved_timed_fields ?? [] as field}
              <p>
                Unresolved timed field: {field.part_key}.{field.field_key}. No value was supplied.
              </p>
            {/each}
            {#if node.incoming_edges.length || node.outgoing_edges.length}
              <p>
                Graph relationships are untimed; their validity at the fictional story time was not
                established.
              </p>
              <ul>
                {#each [...node.incoming_edges, ...node.outgoing_edges] as edge}
                  <li data-bible-edge={edge.edge_id}>
                    <strong>{edge.label}</strong>
                    <p>
                      {nodeName(edge.from_node_id)}
                      {edge.directed ? '→' : '↔'}
                      {nodeName(edge.to_node_id)} ({edgeKindText(edge.edge_kind)})
                    </p>
                  </li>
                {/each}
              </ul>
            {/if}
          </article>
        {/each}
      {/if}
    {/if}
  </section>
</details>

<style>
  section {
    overflow-wrap: anywhere;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
