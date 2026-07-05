import { TerminalHero } from "@/components/hero/TerminalHero";
import { InstallSnippet } from "@/components/install/InstallSnippet";
import { CommandReference } from "@/components/commands/CommandReference";

export function HomePage() {
  return (
    <>
      <TerminalHero />
      <InstallSnippet />
      <CommandReference />
    </>
  );
}
