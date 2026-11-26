"use client";

import { motion } from "motion/react";
import { Coins, Brain, Landmark, Network, Send } from "lucide-react";
import { Badge, Card, CardContent } from "@kian/ui";
import { SUBSYSTEMS, type Subsystem } from "@/lib/kian-data";

const ICONS: Record<Subsystem["icon"], typeof Coins> = {
  coins: Coins,
  brain: Brain,
  landmark: Landmark,
  network: Network,
  channel: Send,
};

export function SubsystemGrid() {
  return (
    <section id="subsystems" className="scroll-mt-24 py-20">
      <div className="mx-auto max-w-6xl px-4 sm:px-6">
        <div className="mb-10 text-center">
          <h2 className="text-3xl font-black sm:text-4xl">
            The five subsystems of the <span className="text-royal-400">Kingdom</span>
          </h2>
          <p className="mx-auto mt-3 max-w-2xl leading-7 text-zinc-400">
            The ecosystem value splits across five pillars — each weight computed from the analysis of the 18 legacy repos
            already computed.
          </p>
        </div>

        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {SUBSYSTEMS.map((s, i) => {
            const Icon = ICONS[s.icon];
            return (
              <motion.div
                key={s.id}
                initial={{ opacity: 0, y: 24 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: "-80px" }}
                transition={{ duration: 0.5, delay: i * 0.08 }}
                className={i === 0 ? "sm:col-span-2 lg:col-span-2" : ""}
              >
                <Card
                  className={`group relative h-full overflow-hidden border-royal-500/20 bg-zinc-900/70 backdrop-blur transition-colors hover:border-gold-400/40 ${
                    i === 0 ? "bg-gradient-to-bl from-royal-600/15 to-zinc-900/70" : ""
                  }`}
                >
                  <CardContent className="flex h-full flex-col gap-4 p-6">
                    <div className="flex items-start justify-between gap-3">
                      <span className="flex size-12 items-center justify-center rounded-2xl bg-royal-500/15 ring-1 ring-royal-500/30 transition-colors group-hover:bg-gold-400/10 group-hover:ring-gold-400/30">
                        <Icon className="size-6 text-royal-400 transition-colors group-hover:text-gold-400" aria-hidden />
                      </span>
                      <Badge className="border-gold-400/40 bg-gold-400/10 text-gold-400 hover:bg-gold-400/10">
                        weight {s.weight}%
                      </Badge>
                    </div>
                    <div>
                      <h3 className="text-xl font-bold">{s.title}</h3>
                      <p className="mt-1 font-mono text-xs text-royal-400" dir="ltr">
                        {s.crates}
                      </p>
                    </div>
                    <p className="flex-1 leading-7 text-zinc-400">{s.description}</p>
                    <div className="flex flex-wrap gap-1.5">
                      {s.stack.map((t) => (
                        <Badge
                          key={t}
                          variant="secondary"
                          className="border-zinc-700/60 bg-zinc-800/80 font-mono text-[11px] text-zinc-300"
                        >
                          {t}
                        </Badge>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </motion.div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
