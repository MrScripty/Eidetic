<script lang="ts">
	import type { Entity, EntityCategory } from '$lib/types.js';
	import { colorToHex } from '$lib/types.js';

	type ScriptElement =
		| { type: 'scene_heading'; text: string }
		| { type: 'action'; text: string }
		| { type: 'character'; text: string }
		| { type: 'parenthetical'; text: string }
		| { type: 'dialogue'; text: string }
		| { type: 'transition'; text: string };

	const HIGHLIGHT_CATEGORIES: EntityCategory[] = ['Character', 'Location', 'Prop'];

	let { text, streaming = false, entities = [] }: {
		text: string;
		streaming?: boolean;
		entities?: Entity[];
	} = $props();

	let elements: ScriptElement[] = $derived(parseScriptText(text));

	/** Entities filtered to highlightable categories. */
	let highlightEntities = $derived(
		entities.filter(e => HIGHLIGHT_CATEGORIES.includes(e.category))
	);

	/** Regex matching any entity name (longest first, case-insensitive, word-boundary). */
	let entityRegex = $derived.by(() => {
		if (highlightEntities.length === 0) return null;
		const names = highlightEntities
			.map(e => e.name)
			.sort((a, b) => b.length - a.length)
			.map(escapeRegex);
		return new RegExp(`\\b(${names.join('|')})\\b`, 'gi');
	});

	/** Fast lookup from lowercase name → entity. */
	let entityMap = $derived.by(() => {
		const map = new Map<string, Entity>();
		for (const e of highlightEntities) {
			map.set(e.name.toLowerCase(), e);
		}
		return map;
	});

	function escapeRegex(s: string): string {
		return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	function escapeHtml(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
	}

	/** Return HTML string with entity names wrapped in colored spans. */
	function highlightText(raw: string): string {
		if (!entityRegex) return escapeHtml(raw);
		let result = '';
		let lastIndex = 0;
		entityRegex.lastIndex = 0;
		let match: RegExpExecArray | null;
		while ((match = entityRegex.exec(raw)) !== null) {
			result += escapeHtml(raw.slice(lastIndex, match.index));
			const entity = entityMap.get(match[0].toLowerCase());
			if (entity) {
				const hex = colorToHex(entity.color);
				result += `<span class="entity-hl" style="border-color:${hex}" title="${escapeHtml(entity.category)}: ${escapeHtml(entity.name)}">${escapeHtml(match[0])}</span>`;
			} else {
				result += escapeHtml(match[0]);
			}
			lastIndex = match.index + match[0].length;
		}
		result += escapeHtml(raw.slice(lastIndex));
		return result;
	}

	/**
	 * Client-side screenplay parser. Mirrors the Rust parser in format.rs.
	 * Detects scene headings, character cues, dialogue, parentheticals,
	 * transitions, and action lines.
	 */
	function parseScriptText(raw: string): ScriptElement[] {
		const result: ScriptElement[] = [];
		const lines = raw.split('\n');
		let state: 'start' | 'after_character' | 'in_dialogue' = 'start';
		let dialogueBuf = '';

		for (const line of lines) {
			const trimmed = line.trim();

			if (trimmed === '') {
				if (dialogueBuf) {
					result.push({ type: 'dialogue', text: dialogueBuf.trim() });
					dialogueBuf = '';
				}
				state = 'start';
				continue;
			}

			if (state === 'start') {
				if (isSceneHeading(trimmed)) {
					result.push({ type: 'scene_heading', text: trimmed });
				} else if (isTransition(trimmed)) {
					result.push({ type: 'transition', text: trimmed });
				} else if (isCharacterCue(trimmed)) {
					result.push({ type: 'character', text: trimmed });
					state = 'after_character';
				} else {
					result.push({ type: 'action', text: trimmed });
				}
			} else if (state === 'after_character') {
				if (isParenthetical(trimmed)) {
					const inner = trimmed.slice(1, -1);
					result.push({ type: 'parenthetical', text: inner });
				} else {
					dialogueBuf = trimmed;
					state = 'in_dialogue';
				}
			} else if (state === 'in_dialogue') {
				if (isParenthetical(trimmed)) {
					if (dialogueBuf) {
						result.push({ type: 'dialogue', text: dialogueBuf.trim() });
						dialogueBuf = '';
					}
					const inner = trimmed.slice(1, -1);
					result.push({ type: 'parenthetical', text: inner });
				} else if (isCharacterCue(trimmed)) {
					if (dialogueBuf) {
						result.push({ type: 'dialogue', text: dialogueBuf.trim() });
						dialogueBuf = '';
					}
					result.push({ type: 'character', text: trimmed });
					state = 'after_character';
				} else {
					if (dialogueBuf) dialogueBuf += ' ';
					dialogueBuf += trimmed;
				}
			}
		}

		if (dialogueBuf) {
			result.push({ type: 'dialogue', text: dialogueBuf.trim() });
		}

		return result;
	}

	function isSceneHeading(line: string): boolean {
		const upper = line.toUpperCase();
		return upper.startsWith('INT.') || upper.startsWith('EXT.') || upper.startsWith('INT/EXT.');
	}

	function isTransition(line: string): boolean {
		const upper = line.trim().toUpperCase();
		return upper.endsWith('TO:') && upper.length <= 30;
	}

	function isCharacterCue(line: string): boolean {
		const trimmed = line.trim();
		if (!trimmed || trimmed.length > 40) return false;
		const namePart = trimmed.includes('(') ? trimmed.slice(0, trimmed.indexOf('(')).trim() : trimmed;
		if (!namePart) return false;
		const alphaChars = namePart.replace(/[^a-zA-Z]/g, '');
		if (!alphaChars) return false;
		const upperCount = (alphaChars.match(/[A-Z]/g) || []).length;
		return upperCount / alphaChars.length > 0.8;
	}

	function isParenthetical(line: string): boolean {
		const trimmed = line.trim();
		return trimmed.startsWith('(') && trimmed.endsWith(')');
	}
</script>

<div class="script-view" class:streaming>
	{#each elements as el}
		{#if highlightEntities.length > 0}
			{#if el.type === 'scene_heading'}
				<p class="el scene-heading">{@html highlightText(el.text)}</p>
			{:else if el.type === 'action'}
				<p class="el action">{@html highlightText(el.text)}</p>
			{:else if el.type === 'character'}
				<p class="el character">{@html highlightText(el.text)}</p>
			{:else if el.type === 'parenthetical'}
				<p class="el parenthetical">({@html highlightText(el.text)})</p>
			{:else if el.type === 'dialogue'}
				<p class="el dialogue">{@html highlightText(el.text)}</p>
			{:else if el.type === 'transition'}
				<p class="el transition">{@html highlightText(el.text)}</p>
			{/if}
		{:else}
			{#if el.type === 'scene_heading'}
				<p class="el scene-heading">{el.text}</p>
			{:else if el.type === 'action'}
				<p class="el action">{el.text}</p>
			{:else if el.type === 'character'}
				<p class="el character">{el.text}</p>
			{:else if el.type === 'parenthetical'}
				<p class="el parenthetical">({el.text})</p>
			{:else if el.type === 'dialogue'}
				<p class="el dialogue">{el.text}</p>
			{:else if el.type === 'transition'}
				<p class="el transition">{el.text}</p>
			{/if}
		{/if}
	{/each}
	{#if streaming}
		<span class="cursor-blink">|</span>
	{/if}
</div>

<style>
	.script-view {
		font-family: 'Courier New', 'Courier', monospace;
		font-size: 0.8rem;
		line-height: 1.5;
		padding: 12px 16px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		overflow: auto;
		color: var(--color-text-primary);
	}

	.streaming {
		border-color: var(--color-status-generating);
	}

	.el {
		margin: 0;
		padding: 0;
	}

	.scene-heading {
		text-transform: uppercase;
		font-weight: bold;
		margin-top: 1.5em;
	}

	.scene-heading:first-child {
		margin-top: 0;
	}

	.action {
		margin-top: 1em;
		max-width: 60ch;
	}

	.character {
		text-transform: uppercase;
		margin-top: 1em;
		padding-left: 15ch;
	}

	.parenthetical {
		padding-left: 11ch;
		font-style: italic;
	}

	.dialogue {
		padding-left: 8ch;
		max-width: 35ch;
	}

	.transition {
		text-transform: uppercase;
		text-align: right;
		margin-top: 1em;
	}

	.script-view :global(.entity-hl) {
		border-bottom: 2px solid;
		cursor: default;
	}

	.cursor-blink {
		animation: blink 1s step-end infinite;
		color: var(--color-accent);
		font-weight: bold;
	}

	@keyframes blink {
		50% { opacity: 0; }
	}
</style>
