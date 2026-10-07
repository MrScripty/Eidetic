import { type Plugin } from '../ui/node_modules/vite/dist/node/index.js';
import { fileURLToPath } from 'node:url';
import base from '../ui/vite.config.js';

const qualifier = (file: string) =>
  fileURLToPath(new URL(`../ui/src/qualification/${file}`, import.meta.url));

/** Explicit test-build seams; production component, store and transport stay byte-identical. */
const lifecycle: Plugin = {
  name: 'qualifier-only-recall-lifecycle',
  enforce: 'pre',
  resolveId(source, importer) {
    if (source === './BibleRecall.svelte' && importer?.endsWith('/BibleGraphNodeDetail.svelte')) {
      return qualifier('RecallLifecycleInspector.svelte');
    }
    if ((source === '$lib/projectionApi.js' || source.endsWith('/lib/projectionApi.js')) && importer?.endsWith('/bibleRecallProjection.svelte.ts')) {
      return qualifier('recallLifecycle.svelte.ts');
    }
    return null;
  },
  transform(code, id) {
    if (!id.endsWith('/src/routes/+layout.svelte')) return null;
    const expected = "<script lang=\"ts\">\n  import '../app.css';";
    if (!code.includes(expected)) throw new Error('Frozen root layout does not match QA host admission');
    return code.replace(expected, `${expected}\n  import RecallLifecycleControls from '../qualification/RecallLifecycleControls.svelte';`)
      + '\n<RecallLifecycleControls />\n';
  },
};

export default { ...base, plugins: [lifecycle, ...(base.plugins ?? [])] };
