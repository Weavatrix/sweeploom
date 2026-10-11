import { LegalPage } from "@/components/LegalPage";
import { getTerms } from "@/lib/legal";
import { pageMeta } from "@/lib/meta";
import { repoFile } from "@/lib/site";

export const metadata = pageMeta({
  title: "Terms of Use",
  description:
    "Terms of Use for the SweepLoom software and sweeploom.com: MPL-2.0 governs the code, provided as is, review plans before you confirm, deletions can be permanent.",
  path: "/terms/",
});

export default async function TermsPage() {
  const terms = await getTerms();
  return (
    <LegalPage
      title="Terms of Use"
      effective={terms.effective}
      current="/terms/"
      intro={
        <p>
          The short version: the code is yours to use under MPL-2.0, it comes with no warranty, and anything SweepLoom
          removes, it removes because you confirmed a plan. Read the plan.
        </p>
      }
    >
      <div dangerouslySetInnerHTML={{ __html: terms.html }} />
      <hr />
      <p className="text-sm">
        This page renders{" "}
        <a href={repoFile("TERMS.md")}>TERMS.md</a> from the SweepLoom repository; its Git history is the record of every
        change.
      </p>
    </LegalPage>
  );
}
