import Image from "next/image";

export function Footer() {
  return (
    <footer className="bg-klar-cream px-6 py-16">
      <div className="mx-auto max-w-5xl">
        <div className="grid gap-12 md:grid-cols-4">
          <div className="md:col-span-1">
            <div className="mb-3 flex items-center gap-2.5">
              <Image
                src="/brand/klar-mascot.svg"
                alt="Klar mascot"
                width={36}
                height={36}
                className="hover-wiggle"
              />
              <span className="text-lg font-bold tracking-tight text-klar-deep">
                klar
              </span>
            </div>
            <p className="text-xs leading-relaxed text-klar-deep/50">
              Designed for machines to write.
              <br />
              Built for humans to trust.
            </p>
          </div>

          <div>
            <h4 className="mb-3 font-mono text-xs font-semibold uppercase tracking-widest text-klar-deep/40">
              Resources
            </h4>
            <ul className="space-y-2 text-sm">
              <li>
                <a href="/docs" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Documentation
                </a>
              </li>
              <li>
                <a href="/docs/language-spec" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Language Spec
                </a>
              </li>
              <li>
                <a href="/docs/getting-started" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Getting Started
                </a>
              </li>
              <li>
                <a href="/docs/benchmark" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Benchmark Suite
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="mb-3 font-mono text-xs font-semibold uppercase tracking-widest text-klar-deep/40">
              Community
            </h4>
            <ul className="space-y-2 text-sm">
              <li>
                <a href="https://github.com/klar-lang/klar" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  GitHub
                </a>
              </li>
              <li>
                <a href="https://discord.gg/klar" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Discord
                </a>
              </li>
              <li>
                <a href="https://github.com/klar-lang/klar/discussions" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Discussions
                </a>
              </li>
              <li>
                <a href="/blog" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Blog
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="mb-3 font-mono text-xs font-semibold uppercase tracking-widest text-klar-deep/40">
              Project
            </h4>
            <ul className="space-y-2 text-sm">
              <li>
                <a href="/docs/roadmap" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Roadmap
                </a>
              </li>
              <li>
                <a href="/docs/contributing" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  Contributing
                </a>
              </li>
              <li>
                <a href="/docs/license" className="link-underline text-klar-deep/70 hover:text-klar-deep">
                  License (Apache 2.0)
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="mt-12 flex flex-col items-center justify-between gap-4 border-t-2 border-klar-deep/10 pt-8 sm:flex-row">
          <p className="font-mono text-[11px] text-klar-deep/35">
            &copy; {new Date().getFullYear()} Klar Contributors. Apache 2.0 License.
          </p>
          <p className="font-mono text-[11px] text-klar-deep/25">
            Less code. Fewer errors. Greener compute.
          </p>
        </div>
      </div>
    </footer>
  );
}
