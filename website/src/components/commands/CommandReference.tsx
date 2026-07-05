import { commands } from "@/data/commands";
import { useState } from "react";
import { CommandTerminalList } from "./CommandTerminalList";
import { StorageLivePanel } from "./StorageLivePanel";

export function CommandReference() {
  const [active, setActive] = useState(commands[0]);

  return (
    <section className="mx-auto max-w-5xl px-6 py-16">
      <h2 className="mb-2 text-2xl font-semibold">Commands</h2>
      <p className="mb-10 text-muted-foreground">
        Select a command to see what it does to your encrypted store.
      </p>
      <div className="overflow-hidden rounded-lg border border-border/50 shadow-xl lg:grid lg:grid-cols-2">
        <StorageLivePanel command={active} />
        <CommandTerminalList
          commands={commands}
          active={active}
          onSelect={setActive}
        />
      </div>
    </section>
  );
}
