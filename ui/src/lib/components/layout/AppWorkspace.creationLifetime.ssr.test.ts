import { afterEach, expect, it, vi } from 'vitest';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';

const factory = vi.hoisted(() => ({ create: vi.fn() }));
vi.mock('../editor/scriptBlockCreationDraft.svelte.js', async () => {
  const actual = await vi.importActual<
    typeof import('../editor/scriptBlockCreationDraft.svelte.js')
  >('../editor/scriptBlockCreationDraft.svelte.js');
  factory.create.mockImplementation(actual.createScriptBlockCreationDraft);
  return { ...actual, createScriptBlockCreationDraft: factory.create };
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  factory.create.mockClear();
});

const source: SelectedNodeEditorNode = {
  node_id: 'scene.original',
  name: 'Original scene',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  locked: false,
  content_status: 'Empty',
};

it('restores the exact uncertain submission when a fresh Script panel consumer returns from Graph and Split', async () => {
  const { render } = await import('svelte/server');
  const invoke = vi
    .fn()
    .mockRejectedValueOnce(new Error('acknowledgement lost after commit'))
    .mockResolvedValueOnce({
      outcome: 'already_recorded',
      projection: {
        version: 5,
        payload: {
          document: { id: 'script.document.main', title: 'Story', sort_order: 0 },
          segments: [],
        },
      },
    });
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  const { default: AppWorkspace } = await import('./AppWorkspace.svelte');
  const initial = render(AppWorkspace, { props: { workspaceMode: 'script' } });
  expect(initial.body).toContain('Write screenplay');
  expect(factory.create).toHaveBeenCalled();
  const original = factory.create.mock.results[0]?.value as ReturnType<
    typeof import('../editor/scriptBlockCreationDraft.svelte.js').createScriptBlockCreationDraft
  >;
  original.begin(source);
  original.state.text = '  Original submission — 雨\n\n';
  await original.save();
  const submitted = structuredClone(invoke.mock.calls[0]);
  expect(original.state.uncertain).toBe(true);
  expect(render(AppWorkspace, { props: { workspaceMode: 'graph' } }).body).not.toContain(
    'Write screenplay',
  );
  expect(render(AppWorkspace, { props: { workspaceMode: 'split' } }).body).not.toContain(
    'Write screenplay',
  );
  const returned = render(AppWorkspace, { props: { workspaceMode: 'script' } }).body;
  expect(returned).toContain('Retry same save');
  expect(returned).toContain('Original submission — 雨');
  expect(returned).toContain('Writing for');
  expect(returned).toContain('Original scene');
  expect(factory.create).toHaveBeenCalledTimes(1);
  await original.save();
  expect(invoke.mock.calls[1]).toEqual(submitted);
  expect(original.state.writing).toBe(false);
  expect(render(AppWorkspace, { props: { workspaceMode: 'script' } }).body).not.toContain(
    'Retry same save',
  );
}, 15000);

it('retains an in-flight save when its acknowledgement fails after the Script consumer disappears', async () => {
  const { render } = await import('svelte/server');
  let loseAcknowledgement!: (error: Error) => void;
  const invoke = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          loseAcknowledgement = reject;
        }),
    )
    .mockResolvedValueOnce({
      outcome: 'already_recorded',
      projection: {
        version: 5,
        payload: {
          document: { id: 'script.document.main', title: 'Story', sort_order: 0 },
          segments: [],
        },
      },
    });
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  const { default: AppWorkspace } = await import('./AppWorkspace.svelte');
  const owner = await import('$lib/stores/scriptBlockCreationSession.svelte.js');
  const modes = await import('$lib/stores/workspaceMode.svelte.js');
  expect(render(AppWorkspace, { props: { workspaceMode: 'script' } }).body).toContain(
    'Write screenplay',
  );
  const original = owner.getSessionScriptBlockCreationDraft();
  original.begin(source);
  original.state.text = 'Draft submitted before navigation';
  const pending = original.save();
  expect(original.state.saving).toBe(true);
  const submitted = structuredClone(invoke.mock.calls[0]);
  for (const mode of ['graph', 'split'] as const) {
    modes.setWorkspaceMode(mode);
    expect(modes.workspaceModeState.mode).toBe(mode);
    expect(
      render(AppWorkspace, { props: { workspaceMode: modes.workspaceModeState.mode } }).body,
    ).not.toContain('Retry same save');
  }
  loseAcknowledgement(new Error('late lost acknowledgement'));
  await pending;
  modes.setWorkspaceMode('script');
  const returned = render(AppWorkspace, {
    props: { workspaceMode: modes.workspaceModeState.mode },
  }).body;
  expect(returned).toContain('Retry same save');
  expect(returned).toContain('Draft submitted before navigation');
  expect(owner.getSessionScriptBlockCreationDraft()).toBe(original);
  expect(factory.create).toHaveBeenCalledTimes(1);
  await owner.getSessionScriptBlockCreationDraft().save();
  expect(invoke.mock.calls[1]).toEqual(submitted);
});

it('replaces the owner on project activation and keeps an old acknowledgement/retry out of the new draft', async () => {
  const { render } = await import('svelte/server');
  let acknowledgeOld!: (value: unknown) => void;
  const invoke = vi.fn().mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        acknowledgeOld = resolve;
      }),
  );
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  const { default: AppWorkspace } = await import('./AppWorkspace.svelte');
  const owner = await import('$lib/stores/scriptBlockCreationSession.svelte.js');
  const { activateProjectSession } = await import('$lib/stores/projectSession.js');
  const { clearScriptDocumentProjection, getCachedScriptDocumentProjection } =
    await import('$lib/stores/scriptDocumentProjection.svelte.js');
  expect(render(AppWorkspace, { props: { workspaceMode: 'script' } }).body).toContain(
    'Write screenplay',
  );
  const original = owner.getSessionScriptBlockCreationDraft();
  original.begin(source);
  original.state.text = 'Old session submission';
  const pending = original.save();
  await activateProjectSession(
    {
      name: 'New session',
      premise: '',
      references: [],
      timeline: {
        total_duration_ms: 1,
        tracks: [],
        nodes: [],
        node_arcs: [],
        relationships: [],
        structure: { template_name: 'Test', segments: [] },
      },
    },
    {
      clearProjectionRefreshQueue: () => {},
      resetEditorState: () => {},
      resetScriptBlockCreationDraft: owner.resetSessionScriptBlockCreationDraft,
      clearBibleSelection: () => {},
      clearProjectionCaches: () =>
        clearScriptDocumentProjection({ document_id: 'script.document.main' }),
      setActiveProject: () => {},
      refreshProjections: async () => {},
    },
  );
  const current = owner.getSessionScriptBlockCreationDraft();
  expect(current).not.toBe(original);
  current.begin({ ...source, name: 'New scene' });
  current.state.text = 'New session draft';
  acknowledgeOld({
    outcome: 'recorded',
    projection: {
      version: 5,
      payload: {
        document: { id: 'script.document.main', title: 'Old project', sort_order: 0 },
        segments: [],
      },
    },
  });
  await pending;
  expect(current.state.writing).toBe(true);
  expect(current.state.text).toBe('New session draft');
  expect(current.state.uncertain).toBe(false);
  expect(
    getCachedScriptDocumentProjection({ document_id: 'script.document.main' }),
  ).toBeUndefined();
  const returned = render(AppWorkspace, { props: { workspaceMode: 'script' } }).body;
  expect(returned).toContain('New session draft');
  expect(returned).not.toContain('Old session submission');
  // Even a retained old consumer cannot admit another request to this session.
  original.begin(source);
  original.state.text = 'Old consumer retry';
  await original.save();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(original.state.error).toBe('The project changed before this screenplay save.');
  expect(current.state.text).toBe('New session draft');
});
