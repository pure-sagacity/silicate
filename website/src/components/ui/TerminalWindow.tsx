import { cn } from "@/lib/utils";

export function TerminalWindow({
  title,
  children,
  className,
  bodyClassName,
}: {
  title?: string;
  children: React.ReactNode;
  className?: string;
  bodyClassName?: string;
}) {
  return (
    <div
      className={cn(
        "flex h-full flex-col overflow-hidden bg-neutral-950",
        className,
      )}
    >
      <div className="flex shrink-0 items-center gap-2 border-b border-border/50 px-4 py-3">
        <span className="size-3 rounded-full bg-red-500/80" />
        <span className="size-3 rounded-full bg-yellow-500/80" />
        <span className="size-3 rounded-full bg-green-500/80" />
        {title && (
          <span className="ml-2 font-mono text-xs text-muted-foreground">
            {title}
          </span>
        )}
      </div>
      <div className={cn("min-h-0 flex-1", bodyClassName)}>{children}</div>
    </div>
  );
}
