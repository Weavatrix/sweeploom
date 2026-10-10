import Link from "next/link";
import { icons } from "@/components/Icons";
import { PageHeader, Section } from "@/components/Section";
import { pageMeta } from "@/lib/meta";
import { SITE } from "@/lib/site";

export const metadata = pageMeta({
  title: "About",
  description:
    "SweepLoom is built by Sergii Ziborov and published by Weavatrix. Local-first, gated apply, live work kept, honest numbers, open source under MPL-2.0.",
  path: "/about/",
});

const PRINCIPLES = [
  {
    title: "Local-first",
    body: "Everything runs on your machine. No account, no cloud, no telemetry. The CLI path carries no HTTP client at all, and the website you are reading has no analytics either.",
  },
  {
    title: "Gated apply",
    body: "Nothing is deleted without plan, revalidate, execute and receipt. People consent with --apply or a confirmation; agents need confirm=true and explicit ids.",
  },
  {
    title: "Keep live work",
    body: "A live agent session is the most valuable thing on the machine, not the biggest. SweepLoom never offers to kill it from the CLI or MCP.",
  },
  {
    title: "Deterministic where it matters",
    body: "No LLM on classify or apply. The path that can remove files is plain, tested Rust that behaves the same every time.",
  },
  {
    title: "Honest numbers",
    body: "Benchmarks are tests that fail, not slides. When a live run found no token saving, that result was published next to the fixtures.",
  },
  {
    title: "Open source",
    body: "SweepLoom is MPL-2.0. You can read, build and change every line that touches your disk.",
  },
];

export default function AboutPage() {
  return (
    <>
      <PageHeader
        eyebrow="About"
        title="A workstation cleaner that understands what you are working on."
        lead="Task Manager shows six node.exe lines. A coding day is a tree of agents, MCP servers, terminals, dev servers and build output. SweepLoom exists to reclaim space from that tree without cutting the branch you are sitting on."
      />

      <Section id="who" eyebrow="Who builds it" title="Sergii Ziborov, for Weavatrix.">
        <div className="grid gap-8 lg:grid-cols-[1.4fr_1fr]">
          <div className="grid gap-4 text-[1.05rem] leading-relaxed text-muted">
            <p>
              SweepLoom is designed and written by <strong className="text-text">Sergii Ziborov</strong> and published by
              Weavatrix. It is developed in the open at{" "}
              <a href={SITE.repo} className="link">
                github.com/Weavatrix/sweeploom
              </a>
              .
            </p>
            <p>
              <strong className="text-text">Weavatrix is the company; SweepLoom is the product.</strong> Weavatrix builds
              open infrastructure for AI software agents: repository evidence graphs, durable memory and safe action
              surfaces. SweepLoom
              applies the same habits, evidence first and explicit gates before any write, to the workstation itself.
            </p>
            <p>
              SweepLoom builds on two MIT-licensed Weavatrix libraries, <code className="inline-code">weavatrix-scan</code> and{" "}
              <code className="inline-code">weavatrix-git</code>, which remain MIT. SweepLoom itself is MPL-2.0.
            </p>
          </div>
          <div className="card grid content-start gap-4 p-6">
            <a href={SITE.companyUrl} className="flex items-center justify-between gap-3 text-sm font-medium hover:text-accent">
              weavatrix.com <icons.external size={15} />
            </a>
            <a href={SITE.companyGitHub} className="flex items-center justify-between gap-3 text-sm font-medium hover:text-accent">
              github.com/Weavatrix <icons.external size={15} />
            </a>
            <a href={SITE.issues} className="flex items-center justify-between gap-3 text-sm font-medium hover:text-accent">
              Questions and bug reports <icons.external size={15} />
            </a>
          </div>
        </div>
      </Section>

      <Section id="principles" eyebrow="Principles" title="What SweepLoom will not trade away." className="border-t border-line">
        <ul className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {PRINCIPLES.map((p) => (
            <li key={p.title} className="card p-6">
              <h3 className="font-display text-lg font-semibold">{p.title}</h3>
              <p className="mt-2 text-[0.95rem] leading-relaxed text-muted">{p.body}</p>
            </li>
          ))}
        </ul>
      </Section>

      <Section id="name" eyebrow="The name" title="A sweep that knows the weave." className="border-t border-line">
        <div className="grid max-w-3xl gap-4 text-[1.05rem] leading-relaxed text-muted">
          <p>
            A workstation is woven: an agent holds an MCP server, which holds a terminal, which started a dev server, which
            built into a folder in a project you are still editing. Sweeping blindly pulls threads. SweepLoom&apos;s mark is a
            loom&apos;s warp and weft inside a ring, and the job is to clear what is loose without unravelling what is in use.
          </p>
          <p>
            Want to help? Read the{" "}
            <Link href="/docs/" className="link">
              docs
            </Link>
            , try it on your own machine, and{" "}
            <a href={SITE.newIssue} className="link">
              open an issue
            </a>{" "}
            when it gets something wrong.
          </p>
        </div>
      </Section>
    </>
  );
}
