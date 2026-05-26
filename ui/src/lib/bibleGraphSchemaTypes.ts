import type {
  BibleGraphNodeCategory,
  BibleGraphNodeId,
  BibleGraphFieldKey,
  BibleGraphPartKey,
  BibleGraphSchemaKey,
} from './bibleGraphTypes.js';

export interface BibleGraphSchemaListProjection {
  schemas: BibleGraphSchemaProjection[];
}

export interface BibleGraphSchemaProjection {
  schema_key: BibleGraphSchemaKey;
  category: BibleGraphNodeCategory;
  display_name: string;
  default_node_name: string;
  canonical_parent_id: BibleGraphNodeId;
  canonical_root_schema_key: BibleGraphSchemaKey;
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
