import { Link } from "react-router-dom";
import { CodeBlock } from "@/components/ui/CodeBlock";

const nixSnippet = `{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    silicate = {
      url = "git+https://github.com/pure-sagacity/silicate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}`;

export function InstallSnippet() {
  return (
    <section className="mx-auto max-w-2xl px-6 py-16">
      <h2 className="mb-2 text-2xl font-semibold">Get started with Nix</h2>
      <p className="mb-6 text-muted-foreground">
        Add Silicate as a flake input in your configuration.
      </p>
      <CodeBlock code={nixSnippet} language="nix" />
      <Link
        to="/install"
        className="mt-6 inline-block text-sm font-medium text-emerald-700 transition-opacity hover:opacity-80"
      >
        All install options →
      </Link>
    </section>
  );
}
