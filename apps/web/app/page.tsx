import { SiteHeader } from "@/components/kian/header";
import { Hero } from "@/components/kian/hero";
import { SubsystemGrid } from "@/components/kian/subsystems";
import { Journey } from "@/components/kian/journey";
import { Roadmap } from "@/components/kian/roadmap";
import { SecurityREA } from "@/components/kian/security";
import { PwaInstall } from "@/components/kian/pwa-install";
import { SiteFooter } from "@/components/kian/footer";

export default function Home() {
  return (
    <div className="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
      <SiteHeader />
      <main className="flex-1">
        <Hero />
        <SubsystemGrid />
        <Journey />
        <Roadmap />
        <SecurityREA />
        <PwaInstall />
      </main>
      <SiteFooter />
    </div>
  );
}
