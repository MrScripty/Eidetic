import { describe, expect, it } from 'vitest';
import { pumasUnloadMessage } from './pumasLifecycle.js';

describe('Pumas unload acknowledgement', () => {
  it.each([
    'ONNX Runtime could not unload the selected model',
    'Ollama could not unload the selected model',
  ])('preserves the provider error instead of claiming unload: %s', (error) => {
    expect(() => pumasUnloadMessage({ success: true, error, unloaded: false })).toThrow(error);
  });

  it('describes an already-unserved model without claiming an unload happened', () => {
    expect(pumasUnloadMessage({ success: true, unloaded: false })).toBe(
      'Selected model was already not served by Pumas',
    );
  });

  it('reports an unload only after its actual acknowledgement', () => {
    expect(pumasUnloadMessage({ success: true, unloaded: true })).toBe(
      'Selected model unloaded through Pumas',
    );
  });
});
