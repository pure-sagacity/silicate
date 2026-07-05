# Silicate website

Marketing and documentation site for [Silicate](https://github.com/pure-sagacity/silicate), a command-line password manager written in Rust.

The site introduces Silicate, walks through installation options, and provides an interactive command reference that shows how each CLI command affects the encrypted local store.

## Pages

| Route | Description |
|-------|-------------|
| `/` | Landing page with animated terminal hero, quick install snippet, and interactive command reference |
| `/install` | Detailed installation guides for Nix, Cargo, and manual builds |

## Tech stack

- [Bun](https://bun.com) — runtime, bundler, and dev server
- [React 19](https://react.dev) with [React Router](https://reactrouter.com)
- [Tailwind CSS 4](https://tailwindcss.com) via `bun-plugin-tailwind`
- [Framer Motion](https://www.framer.com/motion/) for animations
- [Radix UI](https://www.radix-ui.com) + [shadcn/ui](https://ui.shadcn.com) components

There is no Vite or Webpack setup. Bun serves HTML entry points directly and bundles TypeScript, JSX, and CSS on the fly.

## Prerequisites

- [Bun](https://bun.com) v1.3+

If you use [devenv](https://devenv.sh) from the repository root, Bun is included in the shell environment.

## Getting started

From the `website/` directory:

```bash
bun install
bun dev
```

The dev server starts with hot module reloading. Open the URL printed in the terminal (typically `http://localhost:3000`).

## Scripts

| Command | Description |
|---------|-------------|
| `bun dev` | Start the development server with HMR |
| `bun start` | Run the production server (`NODE_ENV=production`) |
| `bun run build` | Build static assets to `dist/` |

## Production build

```bash
bun run build
```

This bundles all HTML entry points under `src/`, minifies output, and writes files to `dist/`. Serve that directory with any static file host or reverse proxy.

To preview locally after building:

```bash
bun start
```

## Project structure

```
website/
├── src/
│   ├── index.ts          # Bun.serve() entry point
│   ├── index.html        # HTML shell
│   ├── frontend.tsx      # React mount point
│   ├── App.tsx           # Routes
│   ├── pages/            # HomePage, InstallPage
│   ├── components/       # UI, layout, hero, install, commands
│   └── data/
│       └── commands.ts   # CLI command metadata for the reference panel
├── styles/
│   └── globals.css       # Tailwind / shadcn theme tokens
├── build.ts              # Production build script
└── components.json       # shadcn/ui configuration
```

Path aliases use `@/*` → `./src/*` (see `tsconfig.json`).

## Related

- [Silicate CLI](../README.md) — password manager source, usage, and security notes
- [Repository](https://github.com/pure-sagacity/silicate)
