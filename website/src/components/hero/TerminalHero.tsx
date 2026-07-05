import { TerminalWindow } from "@/components/ui/TerminalWindow";
import { motion } from "framer-motion";
import { useEffect, useState } from "react";

const WORD = "silicate";
const TYPING_SPEED = 120;

export function TerminalHero() {
  const [typed, setTyped] = useState("");
  const [done, setDone] = useState(false);

  useEffect(() => {
    if (typed.length >= WORD.length) {
      setDone(true);
      return;
    }
    const timer = setTimeout(() => {
      setTyped(WORD.slice(0, typed.length + 1));
    }, TYPING_SPEED);
    return () => clearTimeout(timer);
  }, [typed]);

  return (
    <section className="flex min-h-[calc(100vh-4rem)] items-center justify-center px-6 py-20">
      <motion.div
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: "easeOut" }}
        className="w-full max-w-2xl"
      >
        <div className="overflow-hidden rounded-lg border border-border/50 shadow-2xl">
          <TerminalWindow title="~ silicate">
            <div className="px-6 py-8 font-mono text-lg sm:text-xl">
              <span className="text-emerald-700">$</span>{" "}
              <span className="text-neutral-100">{typed}</span>
              <motion.span
                animate={{ opacity: [1, 0] }}
                transition={{
                  duration: 0.8,
                  repeat: Infinity,
                  repeatType: "reverse",
                }}
                className="ml-0.5 inline-block h-5 w-2 bg-emerald-700 align-middle"
              />
            </div>
          </TerminalWindow>
        </div>

        <motion.p
          initial={{ opacity: 0, y: 12 }}
          animate={done ? { opacity: 1, y: 0 } : { opacity: 0, y: 12 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="mt-8 text-center text-lg text-muted-foreground sm:text-xl"
        >
          A simple password manager,{" "}
          <span className="text-emerald-700">built for speed</span>.
        </motion.p>
      </motion.div>
    </section>
  );
}
