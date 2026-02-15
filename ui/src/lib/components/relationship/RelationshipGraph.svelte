<script lang="ts">
	import type { EntityId } from '$lib/types.js';

	export interface GraphNode {
		id: EntityId;
		name: string;
		color: string;
		x: number;
		y: number;
	}

	export interface GraphEdge {
		sourceId: EntityId;
		targetId: EntityId;
		label: string;
		color: string;
	}

	let {
		nodes,
		edges,
		selectedNodeId = null,
		hoveredNodeId = null,
		onselect,
		onhover,
	}: {
		nodes: GraphNode[];
		edges: GraphEdge[];
		selectedNodeId: string | null;
		hoveredNodeId: string | null;
		onselect: (id: string) => void;
		onhover: (id: string | null) => void;
	} = $props();

	const nodeMap = $derived(new Map(nodes.map(n => [n.id, n])));

	const svgWidth = $derived(
		nodes.length > 0 ? Math.max(...nodes.map(n => n.x)) + 80 : 200
	);
	const svgHeight = $derived(
		nodes.length > 0 ? Math.max(...nodes.map(n => n.y)) + 80 : 200
	);

	/** Set of entity IDs connected to the hovered node (including itself). */
	const connectedToHover = $derived.by(() => {
		if (!hoveredNodeId) return null;
		const set = new Set<string>([hoveredNodeId]);
		for (const edge of edges) {
			if (edge.sourceId === hoveredNodeId) set.add(edge.targetId);
			if (edge.targetId === hoveredNodeId) set.add(edge.sourceId);
		}
		return set;
	});

	function isNodeDimmed(id: string): boolean {
		if (!connectedToHover) return false;
		return !connectedToHover.has(id);
	}

	function isEdgeDimmed(edge: GraphEdge): boolean {
		if (!connectedToHover) return false;
		return !connectedToHover.has(edge.sourceId) || !connectedToHover.has(edge.targetId);
	}

	function edgeMidX(edge: GraphEdge): number {
		const s = nodeMap.get(edge.sourceId);
		const t = nodeMap.get(edge.targetId);
		return s && t ? (s.x + t.x) / 2 : 0;
	}

	function edgeMidY(edge: GraphEdge): number {
		const s = nodeMap.get(edge.sourceId);
		const t = nodeMap.get(edge.targetId);
		return s && t ? (s.y + t.y) / 2 : 0;
	}
</script>

<svg
	class="graph-svg"
	viewBox="0 0 {svgWidth} {svgHeight}"
	xmlns="http://www.w3.org/2000/svg"
>
	<!-- Edges -->
	{#each edges as edge (edge.sourceId + '|' + edge.targetId + '|' + edge.label)}
		{@const source = nodeMap.get(edge.sourceId)}
		{@const target = nodeMap.get(edge.targetId)}
		{#if source && target}
			<g class="edge-group" class:dimmed={isEdgeDimmed(edge)}>
				<line
					x1={source.x} y1={source.y}
					x2={target.x} y2={target.y}
					stroke={edge.color}
					stroke-width="1.5"
					opacity="0.5"
				/>
				<text
					x={edgeMidX(edge)}
					y={edgeMidY(edge) - 6}
					text-anchor="middle"
					fill="var(--color-text-muted)"
					font-size="9"
					class="edge-label"
				>{edge.label}</text>
			</g>
		{/if}
	{/each}

	<!-- Nodes -->
	{#each nodes as node (node.id)}
		<g
			class="node-group"
			class:selected={selectedNodeId === node.id}
			class:dimmed={isNodeDimmed(node.id)}
			transform="translate({node.x}, {node.y})"
			onclick={() => onselect(node.id)}
			onmouseenter={() => onhover(node.id)}
			onmouseleave={() => onhover(null)}
			role="button"
			tabindex="0"
		>
			<circle
				r={selectedNodeId === node.id ? 22 : 18}
				fill={node.color}
				stroke={selectedNodeId === node.id ? 'var(--color-accent)' : 'var(--color-border-default)'}
				stroke-width={selectedNodeId === node.id ? 2 : 1}
				opacity="0.9"
			/>
			<text
				y="30"
				text-anchor="middle"
				fill="var(--color-text-primary)"
				font-size="11"
				class="node-label"
			>{node.name}</text>
		</g>
	{/each}
</svg>

<style>
	.graph-svg {
		width: 100%;
		height: 100%;
	}

	.node-group {
		cursor: pointer;
		transition: opacity 0.15s;
	}

	.node-group.dimmed {
		opacity: 0.25;
	}

	.node-group.selected circle {
		filter: drop-shadow(0 0 6px var(--color-accent));
	}

	.node-label {
		pointer-events: none;
		user-select: none;
	}

	.edge-group {
		transition: opacity 0.15s;
	}

	.edge-group.dimmed {
		opacity: 0.1;
	}

	.edge-label {
		pointer-events: none;
		user-select: none;
	}
</style>
