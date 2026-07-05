export type StorageOp = {
  type: "read" | "write" | "delete";
  detail: string;
};

export type Command = {
  name: string;
  usage: string;
  description: string;
  storage: StorageOp[];
};

export const commands: Command[] = [
  {
    name: "TUI",
    usage: "silicate",
    description: "Interactive browser for stored passwords.",
    storage: [
      { type: "read", detail: "Scan ~/.silicate/*.bin" },
      { type: "read", detail: "Decrypt selected entry" },
    ],
  },
  {
    name: "init",
    usage: "silicate init",
    description: "Initialize the password store.",
    storage: [
      { type: "write", detail: "Create ~/.silicate/ directory" },
      { type: "write", detail: "Store key in system keyring or salt.bin" },
      { type: "write", detail: "Write init_timestamp.txt" },
      { type: "delete", detail: "Wipe existing store on re-init" },
    ],
  },
  {
    name: "insert",
    usage: "silicate insert <website>",
    description: "Store a new password.",
    storage: [
      { type: "read", detail: "Load encryption key from keyring" },
      { type: "write", detail: "Write encrypted {website}.bin" },
    ],
  },
  {
    name: "show",
    usage: "silicate show <website>",
    description: "Display or copy a password to clipboard.",
    storage: [
      { type: "read", detail: "Read {website}.bin" },
      { type: "read", detail: "Decrypt entry with keyring key" },
    ],
  },
  {
    name: "delete",
    usage: "silicate delete <website>",
    description: "Remove a stored password.",
    storage: [
      { type: "read", detail: "Locate matching .bin file" },
      { type: "delete", detail: "Remove {website}.bin from store" },
    ],
  },
  {
    name: "edit",
    usage: "silicate edit <website>",
    description: "Update a password in $EDITOR.",
    storage: [
      { type: "read", detail: "Decrypt entry for editor pre-fill" },
      { type: "write", detail: "Rewrite encrypted .bin file" },
      { type: "write", detail: "Temp file in /tmp (then deleted)" },
    ],
  },
  {
    name: "rename",
    usage: "silicate rename <old> <new>",
    description: "Rename a stored entry.",
    storage: [
      { type: "read", detail: "Locate source .bin file" },
      { type: "write", detail: "Rename to {new_website}.bin on disk" },
    ],
  },
  {
    name: "search",
    usage: "silicate search",
    description: "Search entries interactively with fzf.",
    storage: [
      { type: "read", detail: "List all password files" },
      { type: "read", detail: "Decrypt selected entry" },
    ],
  },
  {
    name: "generate",
    usage: "silicate generate [website]",
    description: "Generate a password, optionally save it.",
    storage: [
      { type: "write", detail: "Write {website}.bin if website given" },
    ],
  },
  {
    name: "list",
    usage: "silicate list",
    description: "List stored entries, optionally filtered by tag.",
    storage: [
      { type: "read", detail: "Scan ~/.silicate/*.bin filenames" },
    ],
  },
  {
    name: "tag list",
    usage: "silicate tag list",
    description: "List all tags from stored entries.",
    storage: [
      { type: "read", detail: "Parse tags from {website}-{tag}.bin names" },
    ],
  },
  {
    name: "stats",
    usage: "silicate stats",
    description: "Show password statistics.",
    storage: [
      { type: "read", detail: "Count password files" },
      { type: "read", detail: "Read init_timestamp.txt" },
    ],
  },
  {
    name: "export",
    usage: "silicate export",
    description: "Export secrets or encryption key to file.",
    storage: [
      { type: "read", detail: "Read all .bin files or keyring key" },
      { type: "write", detail: "Write export JSON or key file" },
    ],
  },
  {
    name: "import",
    usage: "silicate import <file>",
    description: "Import secrets or encryption key from file.",
    storage: [
      { type: "read", detail: "Read JSON or key file" },
      { type: "write", detail: "Create/overwrite .bin files or keyring" },
    ],
  },
];
