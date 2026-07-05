import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Check, Copy } from "lucide-react";
import { useState } from "react";

export function CodeBlock({
  code,
  language,
  className,
}: {
  code: string;
  language?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API can be unavailable or blocked by permissions / insecure context
    }
  }

  return (
    <div className={cn("relative group", className)}>
      {language && (
        <span className="absolute left-4 top-3 text-xs text-muted-foreground">
          {language}
        </span>
      )}
      <Button
        variant="ghost"
        size="icon"
        className="absolute right-2 top-2 size-8 opacity-0 transition-opacity group-hover:opacity-100"
        onClick={handleCopy}
        aria-label="Copy code"
      >
        {copied ? (
          <Check className="size-4 text-emerald-700" />
        ) : (
          <Copy className="size-4" />
        )}
      </Button>
      <pre className="overflow-x-auto rounded-lg border border-border/50 bg-neutral-950 p-4 pt-8 font-mono text-sm leading-relaxed text-neutral-300">
        <code>{code}</code>
      </pre>
    </div>
  );
}
