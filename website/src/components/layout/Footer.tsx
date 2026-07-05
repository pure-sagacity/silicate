const links = [
  { label: "Contact", href: "#" },
  { label: "Ko-fi", href: "#" },
  { label: "GitHub", href: "#" },
];

export function Footer() {
  return (
    <footer className="border-t border-border/50 py-10">
      <div className="mx-auto flex max-w-5xl flex-col items-center gap-4 px-6 sm:flex-row sm:justify-between">
        <p className="text-sm text-muted-foreground">
          Silicate — a simple password manager, built for speed.
        </p>
        <nav className="flex gap-6">
          {links.map((link) => (
            <a
              key={link.label}
              href={link.href}
              className="text-sm text-muted-foreground transition-colors hover:text-emerald-700"
            >
              {link.label}
            </a>
          ))}
        </nav>
      </div>
    </footer>
  );
}
