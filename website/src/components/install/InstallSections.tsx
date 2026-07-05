import { motion } from "framer-motion";
import { CodeBlock } from "@/components/ui/CodeBlock";

const sections = [
  {
    id: "nix",
    title: "Nix",
    description:
      "Add Silicate as a flake input, then reference packages.default in your system configuration.",
    code: `{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    silicate = {
      url = "git+https://github.com/pure-sagacity/silicate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}`,
    language: "nix",
    note: "The flake provides packages.default for x86_64-linux, aarch64-linux, x86_64-darwin, and aarch64-darwin.",
  },
  {
    id: "cargo",
    title: "Cargo",
    description: "Install directly from crates.io with Cargo.",
    code: "cargo install silicate",
    language: "bash",
  },
  {
    id: "manual",
    title: "Manual",
    description:
      "Silicate is written in Rust. You'll need a Rust toolchain, or use devenv shell for a preconfigured environment.",
    code: `git clone https://github.com/pure-sagacity/silicate
cd silicate/
cargo build --release
./target/release/silicate --version`,
    language: "bash",
    note: "Run devenv shell for rustfmt, clippy, and other dev tools.",
  },
];

const navLinks = sections.map((s) => ({ id: s.id, label: s.title }));

export function InstallSections() {
  return (
    <div className="mx-auto max-w-3xl px-6 py-16">
      <h1 className="mb-2 text-3xl font-semibold">Install Silicate</h1>
      <p className="mb-10 text-muted-foreground">
        Choose your preferred installation method.
      </p>

      <nav className="mb-12 flex flex-wrap gap-4 border-b border-border/50 pb-4">
        {navLinks.map((link) => (
          <a
            key={link.id}
            href={`#${link.id}`}
            className="text-sm text-muted-foreground transition-colors hover:text-emerald-700"
          >
            {link.label}
          </a>
        ))}
      </nav>

      <div className="space-y-20">
        {sections.map((section, i) => (
          <motion.section
            key={section.id}
            id={section.id}
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-50px" }}
            transition={{ duration: 0.5, delay: i * 0.05 }}
          >
            <h2 className="mb-2 text-2xl font-semibold">{section.title}</h2>
            <p className="mb-6 text-muted-foreground">{section.description}</p>
            <CodeBlock code={section.code} language={section.language} />
            {section.note && (
              <p className="mt-4 text-sm text-muted-foreground">{section.note}</p>
            )}
          </motion.section>
        ))}
      </div>
    </div>
  );
}
