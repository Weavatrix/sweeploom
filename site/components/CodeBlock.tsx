import { CopyButton } from "./CopyButton";

/* Terminal-styled block. Lines that start with "$ " get a gold prompt that is
   excluded from selection and from the copied text; "# " lines are dimmed.
   `copy` defaults to the commands only (prompt lines) when any exist. */

type Props = {
  code: string;
  title?: string;
  copy?: string | false;
  className?: string;
  /** Soft-wrap long lines instead of scrolling (for one-line commands). */
  wrap?: boolean;
};

const INLINE_COMMENT = /\s{2,}#.*$/;

function renderLine(line: string, index: number) {
  if (line.startsWith("$ ")) {
    const body = line.slice(2);
    const match = body.match(INLINE_COMMENT);
    const command = match ? body.slice(0, match.index) : body;
    return (
      <span key={index} className="block">
        <span className="t-prompt">$ </span>
        <span>{command}</span>
        {match ? <span className="t-dim">{match[0]}</span> : null}
      </span>
    );
  }
  if (/^\s*#/.test(line)) {
    return (
      <span key={index} className="t-dim block">
        {line}
      </span>
    );
  }
  return (
    <span key={index} className="block">
      {line || " "}
    </span>
  );
}

export function CodeBlock({ code, title, copy, className, wrap }: Props) {
  const lines = code.replace(/\n$/, "").replace(/\t/g, "  ").split("\n");
  const commands = lines
    .filter((line) => line.startsWith("$ "))
    .map((line) => line.slice(2).replace(INLINE_COMMENT, ""));
  const copyText = copy === false ? null : copy ?? (commands.length ? commands.join("\n") : code);

  return (
    <div className={"term " + (className ?? "")}>
      <div className="term-bar">
        <span className="term-dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        {title ? <span className="truncate">{title}</span> : null}
        {copyText ? <CopyButton text={copyText} /> : null}
      </div>
      <pre
        tabIndex={0}
        aria-label={title ? `${title} code` : "Code"}
        className={wrap ? "whitespace-pre-wrap [overflow-wrap:anywhere]" : undefined}
      >
        <code>{lines.map(renderLine)}</code>
      </pre>
    </div>
  );
}
