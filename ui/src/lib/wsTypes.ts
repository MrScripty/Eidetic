export type ServerMessage =
  | { type: 'timeline_changed' }
  | { type: 'hierarchy_changed' }
  | { type: 'story_changed' }
  | { type: 'node_updated'; node_id: string }
  | { type: 'generation_context'; node_id: string; system_prompt: string; user_prompt: string }
  | { type: 'generation_progress'; node_id: string; token: string; tokens_generated: number }
  | { type: 'generation_complete'; node_id: string }
  | { type: 'generation_error'; node_id: string; error: string }
  | { type: 'bible_changed' }
  | { type: 'script_changed' }
  | { type: 'diffusion_progress'; node_id: string; step: number; total_steps: number }
  | { type: 'diffusion_complete'; node_id: string }
  | { type: 'diffusion_error'; node_id: string; error: string };
