import { beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import BibleGraphNodeDetail from './BibleGraphNodeDetail.svelte';

const detail = vi.hoisted(() => ({ pending: false, error: 'Initial detail read failed' }));
vi.mock('$lib/stores/bibleGraphNodeProjection.svelte.js', async (importOriginal) => ({
  ...(await importOriginal<typeof import('$lib/stores/bibleGraphNodeProjection.svelte.js')>()),
  getCachedBibleGraphNodeProjection: () => undefined,
  isBibleGraphNodeProjectionPending: () => detail.pending,
  getBibleGraphNodeProjectionError: () => detail.error,
  isBibleGraphNodeProjectionVerified: () => false,
  refreshBibleGraphNodeProjection: vi.fn(),
  retainBibleGraphNodeDetail: vi.fn(),
  deleteBibleGraphNodeProjection: vi.fn(),
  refreshBibleGraphNodeListProjection: vi.fn(),
  setBibleGraphNodeNameProjection: vi.fn(),
}));

beforeEach(() => {
  detail.pending = false;
  detail.error = 'Initial detail read failed';
});

function body() {
  return render(BibleGraphNodeDetail, {
    props: { nodeId: 'Mara', onclose: () => {}, edgeTargetNodes: [] },
  }).body;
}

it('exposes an enabled in-place Retry when the first detail read fails without a cache', () => {
  const html = body();
  expect(html).toContain('role="alert"');
  expect(html).toContain('Initial detail read failed');
  expect(html).toMatch(/<button[^>]*>Retry saved facts<\/button>/);
  expect(html).not.toMatch(/<button[^>]*disabled[^>]*>Retry saved facts<\/button>/);
});

it('shows only Loading while the uncached retry is pending', () => {
  detail.pending = true;
  detail.error = '';
  const html = body();
  expect(html).toContain('role="status"');
  expect(html).toContain('Loading');
  expect(html).not.toContain('Retry saved facts');
  expect(html).not.toContain('Initial detail read failed');
});
