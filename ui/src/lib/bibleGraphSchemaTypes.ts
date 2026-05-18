import type { BibleGraphFieldKey, BibleGraphPartKey, BibleGraphSchemaKey } from './types.js';

export interface BibleGraphSchemaListProjection {
  schemas: BibleGraphSchemaProjection[];
}

export interface BibleGraphSchemaProjection {
  schema_key: BibleGraphSchemaKey;
  parts: BibleGraphPartSchemaProjection[];
}

export interface BibleGraphPartSchemaProjection {
  part_key: BibleGraphPartKey;
  name: string;
  sort_order: number;
  fields: BibleGraphFieldSchemaProjection[];
}

export interface BibleGraphFieldSchemaProjection {
  field_key: BibleGraphFieldKey;
  sort_order: number;
}
