import { createFromRoot } from 'codama';
import { rootNodeFromAnchor, type AnchorIdl } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from "@codama/renderers-js";
import anchorIdl from '../../idl.json';
import path from 'path';

console.log("Generating client...");

const codama = createFromRoot(rootNodeFromAnchor(anchorIdl as AnchorIdl));
const jsClient = path.join(import.meta.dirname, "..", "clients", "js");

codama.accept(renderJavaScriptVisitor(path.join(jsClient, "src","generated")));