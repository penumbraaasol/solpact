// Generates the typed @solana/kit client in clients/js/src/generated from the Anchor IDL.
import { readFileSync } from "node:fs";
import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor } from "@codama/renderers-js";

const idl = JSON.parse(readFileSync("target/idl/pact.json", "utf8"));
const codama = createFromRoot(rootNodeFromAnchor(idl));
await codama.accept(renderVisitor("clients/js"));
console.log("Generated clients/js from target/idl/pact.json");
