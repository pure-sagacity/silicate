import { Link } from "react-router-dom";
import { Footer } from "./Footer";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col">
      <header className="border-b border-border/50">
        <nav className="mx-auto flex max-w-5xl items-center justify-between px-6 py-4">
          <Link
            to="/"
            className="font-mono text-lg font-semibold text-emerald-700 transition-opacity hover:opacity-80"
          >
            silicate
          </Link>
          <Link
            to="/install"
            className="text-sm text-muted-foreground transition-colors hover:text-emerald-700"
          >
            Install
          </Link>
        </nav>
      </header>
      <main className="flex-1">{children}</main>
      <Footer />
    </div>
  );
}
