import { type Plugin } from "../ui/node_modules/vite/dist/node/index.js";
import { fileURLToPath } from "node:url";
import base from "../ui/vite.config.js";

/** Arc Description qualifier uses existing public read/decision seams; application source is frozen. */
const manualFacts: Plugin = {
    name: "qualifier-only-manual-facts",
    enforce: "pre",
    resolveId(source, importer) {
        if ((source === "$lib/api.js" || source.endsWith("/lib/api.js")) && importer?.endsWith("/BeatEditor.svelte")) {
            return fileURLToPath(new URL("../ui/src/qualification/notesPrompt.svelte.ts", import.meta.url));
        }
        if (
            (source === "$lib/commandApi.js" ||
                source.endsWith("/lib/commandApi.js")) &&
            (importer?.endsWith("/propagationProposalProjection.svelte.ts") ||
             importer?.endsWith("/bibleGraphNodeProjection.svelte.ts"))
        ) {
            return fileURLToPath(
                new URL(
                    "../ui/src/qualification/manualFacts.svelte.ts",
                    import.meta.url,
                ),
            );
        }
        if ((source === "$lib/projectionApi.js" || source.endsWith("/lib/projectionApi.js")) && importer?.endsWith("/bibleGraphNodeDetailProjection.svelte.ts")) {
            return fileURLToPath(new URL("../ui/src/qualification/manualFacts.svelte.ts", import.meta.url));
        }
        return null;
    },
    transform(code, id) {
        if (!id.endsWith("/src/routes/+layout.svelte")) return null;
        const expected = "<script lang=\"ts\">\n  import '../app.css';";
        if (!code.includes(expected))
            throw new Error("Frozen root layout does not match QA admission");
        return (
            code.replace(
                expected,
                `${expected}\n  import ManualFactControls from '../qualification/ManualFactControls.svelte';`,
            ) + "\n<ManualFactControls />\n"
        );
    },
};
export default { ...base, plugins: [manualFacts, ...(base.plugins ?? [])] };
