/* A condensed MCP chat: what the person typed, what the agent called, what
   SweepLoom returned, and what the agent said back. Transcribed from
   docs/MCP-CHATS.md and docs/MCP-CHATS-AGENTS.md; nothing here is invented. */

export type ChatTurn =
  | { role: "user"; text: string }
  | { role: "call"; text: string }
  | { role: "tool"; text: string; label?: string }
  | { role: "agent"; text: string };

const LABEL: Record<ChatTurn["role"], string> = {
  user: "You",
  call: "Agent calls",
  tool: "SweepLoom returns",
  agent: "Agent",
};

export function Chat({ title, verdict, turns }: { title: string; verdict?: { text: string; tone: "ok" | "warn" }; turns: ChatTurn[] }) {
  return (
    <figure className="card overflow-hidden p-0">
      <figcaption className="flex flex-wrap items-center justify-between gap-2 border-b border-line bg-bg-2 px-4 py-3 sm:px-5">
        <span className="font-medium">{title}</span>
        {verdict ? <span className={"chip " + (verdict.tone === "ok" ? "chip-ok" : "chip-warn")}>{verdict.text}</span> : null}
      </figcaption>
      <ol className="grid gap-4 p-4 sm:p-5">
        {turns.map((turn, index) => (
          <li key={index} className={"grid gap-1.5 " + (turn.role === "user" ? "justify-items-end" : "justify-items-start")}>
            <span className="font-mono text-[0.66rem] tracking-[0.12em] text-faint uppercase">
              {turn.role === "tool" && turn.label ? turn.label : LABEL[turn.role]}
            </span>
            {turn.role === "user" ? (
              <p className="chat-bubble chat-user">{turn.text}</p>
            ) : turn.role === "agent" ? (
              <p className="chat-bubble chat-agent">{turn.text}</p>
            ) : (
              <pre className={"chat-code " + (turn.role === "call" ? "chat-call" : "")} tabIndex={0}>
                <code>{turn.text}</code>
              </pre>
            )}
          </li>
        ))}
      </ol>
    </figure>
  );
}
