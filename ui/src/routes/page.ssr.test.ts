import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import Page from './+page.svelte';

describe('root page SSR', () => {
  it('does not require Tauri event transport while rendering on the server', () => {
    const rendered = render(Page);

    expect(rendered.head).toContain('<title>Eidetic</title>');
  });
});
