import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('reopens and accepts durable material with actual Svelte client proxies', () => {
  // Ordinary rune tests use the server transform. Compile this controller for
  // the client so picking a reactive saved plan exercises browser proxy rules.
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';

const source = fs.readFileSync('src/lib/components/editor/childPlanReview.svelte.ts', 'utf8');
const js = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
let compiled = compileModule(js, { filename: 'childPlanReview.svelte.js', generate: 'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g,
  JSON.stringify(import.meta.resolve('svelte/internal/client')));
const { createChildPlanReview } = await import('data:text/javascript;base64,' + Buffer.from(compiled).toString('base64'));
const plan = {
  id: 'durable', parent_node_id: 'scene', target_child_level: 'Beat',
  children: [{ name: 'Exact', outline: 'Saved outline', weight: 1, beat_type: null,
    characters: ['Mara'], props: ['Umbrella'], location: null }],
  script_context: [{ text: 'Exact manual text\\n\\n', block_id: 'manual', revision_event_id: 'saved' }],
  bible_context: {
    context: { version: 3, payload: { target_node_id: 'scene', nodes: [{ node_id: 'Mara', name: 'Original Mara', fields: [{ part_key: 'profile', field_key: 'tagline', value: { type: 'text', value: 'Original blue fact' } }] }] } },
    inputs: [{ node_id: 'Mara', part_key: 'profile', field_key: 'tagline', field_id: 'fact', revision_event_id: 'blue-revision', value: { type: 'text', value: 'Original blue fact' } }],
  },
};
const expected = structuredClone(plan);
const calls = [];
const review = createChildPlanReview({
  owner: () => ({ nodeId: 'scene', session: 0 }), mounted: () => true,
  load: async () => ({ payload: { plans: [{ plan, status: 'pending', created_at_ms: 0 }] } }),
  generate: async () => { throw new Error('Recovery must not generate'); },
  apply: async (payload, commandId) => { calls.push({ payload, commandId }); },
  accepted: async () => {},
});
await review.recover();
assert.equal(review.state.plan, null);
assert.equal(calls.length, 0);
plan.children[0].outline = 'Later provider mutation';
plan.bible_context.context.payload.nodes[0].fields[0].value.value = 'Later green fact';
plan.bible_context.inputs[0].revision_event_id = 'green-revision';
review.reviewSaved('durable');
assert.equal(review.state.plan.id, expected.id);
assert.equal(review.state.plan.children[0].outline, expected.children[0].outline);
assert.equal(review.state.plan.script_context[0].text, expected.script_context[0].text);
assert.deepEqual(JSON.parse(JSON.stringify(review.state.plan.bible_context)), expected.bible_context);
assert.equal(calls.length, 0);
await review.accept();
assert.equal(calls.length, 1);
assert.equal(calls[0].payload.child_plan_id, expected.id);
assert.deepEqual(calls[0].payload.children, expected.children);
assert.equal(typeof calls[0].commandId, 'string');
assert.equal(review.state.plan, null);
console.log('Client proxy recovery and explicit acceptance passed');
`,
  });
  expect(output).toContain('Client proxy recovery and explicit acceptance passed');
});
