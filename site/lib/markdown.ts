import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkRehype from "remark-rehype";
import rehypeSlug from "rehype-slug";
import rehypeStringify from "rehype-stringify";

/* Build-time Markdown → HTML (GFM tables, heading ids). Raw HTML in the
   source is not passed through. */

type HNode = {
  type: string;
  tagName?: string;
  value?: string;
  properties?: Record<string, unknown>;
  children?: HNode[];
};

function el(tagName: string, properties: Record<string, unknown>, children: HNode[]): HNode {
  return { type: "element", tagName, properties, children };
}

function languageOf(pre: HNode): string {
  const code = pre.children?.find((child) => child.type === "element" && child.tagName === "code");
  const classes = code?.properties?.className;
  const list = Array.isArray(classes) ? classes.map(String) : [];
  const lang = list.find((name) => name.startsWith("language-"));
  return lang ? lang.slice("language-".length) : "text";
}

/* Wraps every fenced block in the same terminal frame the TSX CodeBlock uses:
   a bar with the language and a Copy button, then a focusable <pre> so
   keyboard users can scroll long lines. The button stays hidden until
   CodeCopy (client) has attached its handler. */
function rehypeCodeFrames() {
  return (tree: unknown) => {
    const walk = (node: HNode) => {
      if (!node.children) return;
      node.children = node.children.map((child) => {
        if (child.type === "element" && child.tagName === "pre") {
          const lang = languageOf(child);
          child.properties = { ...child.properties, tabIndex: 0, ariaLabel: `${lang} code` };
          const bar = el("div", { className: ["code-bar"] }, [
            el("span", { className: ["term-dots"], ariaHidden: "true" }, [el("i", {}, []), el("i", {}, []), el("i", {}, [])]),
            el("span", {}, [{ type: "text", value: lang }]),
            el("button", { type: "button", className: ["code-copy"], ariaLabel: "Copy code to clipboard" }, [
              { type: "text", value: "Copy" },
            ]),
          ]);
          return el("div", { className: ["code-wrap"] }, [bar, child]);
        }
        walk(child);
        return child;
      });
    };
    walk(tree as HNode);
  };
}

export async function markdownToHtml(source: string): Promise<string> {
  const file = await unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkRehype)
    .use(rehypeSlug)
    .use(rehypeCodeFrames)
    .use(rehypeStringify)
    .process(source);
  return String(file);
}
