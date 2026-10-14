import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import AiPromptPreview from './AiPromptPreview.svelte';

describe('consumed generation prompt inspection', () => {
  it('shows the actually retrieved reference text without a refresh that could replace the receipt', () => {
    const { body } = render(AiPromptPreview, {
      props: {
        notes: 'Generate a scene at the secret station',
        context: {
          system: 'Reference material: The station is called Marmalade Harbor.',
          user: 'Write the scene.',
        },
        loading: false,
        label: 'Last Generation Prompt',
        refreshable: false,
      },
    });
    expect(body).toContain('Last Generation Prompt');
    expect(body).toContain('Marmalade Harbor');
    expect(body).not.toContain('<button');
  });
});
