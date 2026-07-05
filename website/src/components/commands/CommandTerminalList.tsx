import type { Command } from "@/data/commands";
import { TerminalWindow } from "@/components/ui/TerminalWindow";
import { cn } from "@/lib/utils";
import { motion } from "framer-motion";

export function CommandTerminalList({
  commands,
  active,
  onSelect,
}: {
  commands: Command[];
  active: Command;
  onSelect: (command: Command) => void;
}) {
  return (
    <TerminalWindow title="~ silicate --help" className="min-h-[320px] lg:min-h-[480px]">
      <div className="overflow-y-auto p-2 font-mono text-sm">
        {commands.map((command) => {
          const isActive = active.name === command.name;
          return (
            <motion.button
              key={command.name}
              type="button"
              onMouseEnter={() => onSelect(command)}
              onClick={() => onSelect(command)}
              className={cn(
                "w-full rounded-r px-3 py-2.5 text-left transition-colors",
                isActive
                  ? "border-l-2 border-emerald-700 bg-emerald-700/10"
                  : "border-l-2 border-transparent hover:bg-white/5",
              )}
              whileTap={{ scale: 0.99 }}
            >
              <span className="text-emerald-700">$</span>{" "}
              <span className={cn("text-neutral-100", isActive && "font-medium")}>
                {command.usage}
              </span>
              <p className="mt-0.5 pl-4 text-xs text-muted-foreground">
                {command.description}
              </p>
            </motion.button>
          );
        })}
      </div>
    </TerminalWindow>
  );
}
