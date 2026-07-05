import type { Command, StorageOp } from "@/data/commands";
import { cn } from "@/lib/utils";
import { AnimatePresence, motion } from "framer-motion";
import { Check } from "lucide-react";

const opStyles: Record<StorageOp["type"], string> = {
  read: "text-sky-400",
  write: "text-emerald-700",
  delete: "text-red-400",
};

const opLabels: Record<StorageOp["type"], string> = {
  read: "Read",
  write: "Write",
  delete: "Delete",
};

const storeNodes = [
  { id: "dir", label: "~/.silicate/", path: "dir" },
  { id: "salt", label: "salt.bin", path: "file" },
  { id: "init", label: "init_timestamp.txt", path: "file" },
  { id: "github", label: "github.bin", path: "file" },
  { id: "keyring", label: "system keyring", path: "keyring" },
];

function getHighlightedNodes(command: Command): Set<string> {
  const highlighted = new Set<string>();
  for (const op of command.storage) {
    const d = op.detail.toLowerCase();
    if (d.includes("keyring")) highlighted.add("keyring");
    if (d.includes("salt.bin")) highlighted.add("salt");
    if (d.includes("init_timestamp")) highlighted.add("init");
    if (d.includes(".bin") || d.includes("directory") || d.includes("store"))
      highlighted.add("dir");
    if (d.includes("{website}") || d.includes("github") || d.includes(".bin"))
      highlighted.add("github");
  }
  return highlighted;
}

function StorageOpRow({ op, index }: { op: StorageOp; index: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -8 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.25, delay: index * 0.08 }}
      className="flex items-start gap-2 font-mono text-xs"
    >
      <motion.span
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: index * 0.08 + 0.15, type: "spring", stiffness: 400 }}
        className="mt-0.5 shrink-0"
      >
        <Check className="size-3 text-emerald-700" />
      </motion.span>
      <span className={cn("shrink-0 font-semibold uppercase", opStyles[op.type])}>
        {opLabels[op.type]}
      </span>
      <span className="text-muted-foreground">{op.detail}</span>
    </motion.div>
  );
}

function PanelContent({ command }: { command: Command }) {
  const highlighted = getHighlightedNodes(command);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="flex h-full flex-col gap-6 p-5 font-mono text-sm"
    >
      <div>
        <p className="text-neutral-100">
          <span className="text-emerald-700">$</span> {command.usage}
        </p>
        <p className="mt-2 text-xs text-muted-foreground">{command.description}</p>
      </div>

      <div className="space-y-2">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Storage activity
        </p>
        {command.storage.length === 0 ? (
          <p className="text-xs text-muted-foreground">No persistence</p>
        ) : (
          <div className="space-y-2">
            {command.storage.map((op, i) => (
              <StorageOpRow key={i} op={op} index={i} />
            ))}
          </div>
        )}
      </div>

      <div className="mt-auto space-y-2">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Store
        </p>
        <div className="space-y-1 rounded border border-border/30 bg-neutral-950/80 p-3 text-xs">
          {storeNodes.map((node) => (
            <motion.div
              key={node.id}
              animate={{
                borderColor: highlighted.has(node.id)
                  ? "rgba(4, 120, 87, 0.8)"
                  : "rgba(255, 255, 255, 0.05)",
                backgroundColor: highlighted.has(node.id)
                  ? "rgba(4, 120, 87, 0.1)"
                  : "transparent",
              }}
              transition={{ duration: 0.3 }}
              className={cn(
                "rounded px-2 py-1",
                node.path === "dir" ? "text-emerald-700" : "pl-4 text-neutral-400",
              )}
            >
              {node.path === "file" && "├── "}
              {node.path === "keyring" && "└── "}
              {node.label}
            </motion.div>
          ))}
        </div>
      </div>
    </motion.div>
  );
}

export function StorageLivePanel({ command }: { command: Command | null }) {
  return (
    <div className="min-h-[320px] bg-neutral-950/50 lg:min-h-[480px] lg:border-r lg:border-border/50">
      <div className="flex h-full flex-col">
        <div className="border-b border-border/50 px-4 py-3">
          <span className="font-mono text-xs text-muted-foreground">~/.silicate</span>
        </div>
        <div className="relative min-h-0 flex-1">
          <AnimatePresence mode="wait">
            {command ? (
              <PanelContent key={command.name} command={command} />
            ) : (
              <motion.p
                key="idle"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="p-5 text-sm text-muted-foreground"
              >
                Hover a command to preview storage effects.
              </motion.p>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
