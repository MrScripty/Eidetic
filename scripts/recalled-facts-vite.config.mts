import { type Plugin } from "../ui/node_modules/vite/dist/node/index.js";
import { fileURLToPath } from "node:url";
import base from "../ui/vite.config.js";

/** Explicit qualifier imports; production application files remain byte-identical. */
const selectedFacts: Plugin = {
    name: "qualifier-only-recalled-facts",
    enforce: "pre",
    resolveId(source, importer) {
        if (
            (source === "$lib/commandApi.js" ||
                source.endsWith("/lib/commandApi.js")) &&
            importer?.endsWith("/propagationProposalProjection.svelte.ts")
        ) {
            return fileURLToPath(
                new URL(
                    "../ui/src/qualification/recalledFacts.svelte.ts",
                    import.meta.url,
                ),
            );
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
                `${expected}\n  import RecalledFactsControls from '../qualification/RecalledFactsControls.svelte';`,
            ) + "\n<RecalledFactsControls />\n"
        );
    },
};
export default { ...base, plugins: [selectedFacts, ...(base.plugins ?? [])] };
