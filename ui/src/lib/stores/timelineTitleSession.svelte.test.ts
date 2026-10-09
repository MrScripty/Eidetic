import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { compileModule } from 'svelte/compiler';
import ts from 'typescript';
import { expect, it, vi } from 'vitest';
import { getSessionTimelineTitleDraft } from './timelineTitleSession.svelte.js';
import { editorState, resetEditorState } from './editor.svelte.js';
import { applyTimelineNodeNameCommand } from './timelineRenderProjection.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';

vi.mock('./timelineRenderProjection.svelte.js', () => ({
  applyTimelineNodeNameCommand: vi.fn().mockResolvedValue({}),
}));
vi.mock('./selectedNodeEditorProjection.svelte.js', () => ({
  refreshSelectedNodeEditorProjection: vi.fn().mockResolvedValue({
    payload: { node: { node_id: 'A', name_read: { name: 'Saved', revision_event_id: 'new' } } },
  }),
}));
vi.mock('./scriptDocumentProjection.svelte.js', () => ({ invalidateScriptContext: vi.fn() }));
const node: SelectedNodeEditorNode = {
  node_id: 'A',
  name: 'A',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  content_status: 'HasContent',
  locked: false,
  name_read: { name: 'A', revision_event_id: 'write' },
};

it('keeps a title intent through panel/selection replacement and excludes old project callers', async () => {
  resetEditorState();
  const first = getSessionTimelineTitleDraft(node);
  first.initialize(node.name_read);
  first.state.name = '  Station departure — 雨.  ';
  getSessionTimelineTitleDraft({ ...node, node_id: 'B' });
  expect(getSessionTimelineTitleDraft({ ...node })).toBe(first);
  expect(first.state.name).toBe('  Station departure — 雨.  ');
  resetEditorState();
  expect(getSessionTimelineTitleDraft(node)).not.toBe(first);
  await first.apply();
  expect(applyTimelineNodeNameCommand).not.toHaveBeenCalled();
  expect(first.state.error).toContain('project changed');
});

it('registers and reuses the actual session owner inside a client derived read', () => {
  // SSR erases client mutation guards. Compile the real owner for the client,
  // stubbing only transport/draft construction to isolate owner registration.
  let source = readFileSync(new URL('./timelineTitleSession.svelte.ts', import.meta.url), 'utf8');
  source = source.replace(/import[\s\S]*?from [^;]+;/g, '');
  source =
    'const untrack = runtime.untrack; function createTimelineTitleDraft() { const state=runtime.state({name:""}); return {state}; } const applyTimelineNodeNameCommand=()=>{}; const refreshSelectedNodeEditorProjection=()=>{}; const getEditorSessionGeneration=()=>1; const editorState={selectedNodeId:"A"}; const invalidateScriptContext=()=>{}; const createCommandId=()=>"title";\n' +
    source;
  const javascript = ts.transpileModule(source, {
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
  }).outputText;
  const compiled = compileModule(javascript, {
    generate: 'client',
    filename: 'timelineTitleSession.svelte.js',
    dev: true,
  }).js.code;
  const body =
    compiled
      .replace(/import \* as \$ from ['"]svelte\/internal\/client['"];?/, 'const $ = runtime;')
      .replace(/export function /g, 'function ') + '\nreturn { getSessionTimelineTitleDraft };';
  const require = createRequire(import.meta.url);
  const script = `import * as runtime from ${JSON.stringify(pathToFileURL(require.resolve('svelte/internal/client')).href)};
    const owner = new Function('runtime', ${JSON.stringify(body)})(runtime);
    runtime.push({}, true);
    try {
      const first = runtime.get(runtime.derived(() => owner.getSessionTimelineTitleDraft({node_id:'A'})));
      const second = runtime.get(runtime.derived(() => owner.getSessionTimelineTitleDraft({node_id:'A'})));
      if (first !== second) throw new Error('client consumer replaced the persistent owner');
    } finally { runtime.pop(); }`;
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', script], {
    encoding: 'utf8',
    timeout: 10_000,
  });
  expect(result.stderr).toBe('');
  expect(result.status).toBe(0);
});

it('acknowledgement directly invalidates context and refreshes only the current selected owner', async () => {
  resetEditorState();
  editorState.selectedNodeId = 'A';
  const draft = getSessionTimelineTitleDraft(node);
  draft.initialize(node.name_read);
  draft.state.name = 'Saved';
  await draft.apply();
  expect(draft.state.saved).toBe(true);
  expect(invalidateScriptContext).toHaveBeenCalled();
  const guard = vi.mocked(refreshSelectedNodeEditorProjection).mock.calls.at(-1)?.[1];
  expect(guard?.()).toBe(true);
  editorState.selectedNodeId = 'B';
  expect(guard?.()).toBe(false);
  resetEditorState();
  editorState.selectedNodeId = 'A';
  expect(guard?.()).toBe(false);
});
