<script lang="ts">
	type ScriptElement =
		| { type: 'scene_heading'; text: string }
		| { type: 'action'; text: string }
		| { type: 'character'; text: string }
		| { type: 'parenthetical'; text: string }
		| { type: 'dialogue'; text: string }
		| { type: 'transition'; text: string };

	let { text, streaming = false }: { text: string; streaming?: boolean } = $props();

	let elements: ScriptElement[] = $derived(parseScriptText(text));

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

	.cursor-blink {
		animation: blink 1s step-end infinite;
		color: var(--color-accent);
		font-weight: bold;
	}

	@keyframes blink {
		50% { opacity: 0; }
	}
</style>
