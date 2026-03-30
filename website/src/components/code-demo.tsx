"use client";

import { motion } from "motion/react";

const klarCode = `@schema
struct User {
    name: String       @min_len(1) @max_len(100)
    email: String      @format(email)
    age: Int           @range(0, 150)
    role: Role = Role.Member
}

fn create_user(req: Request) -> Response ! AppError {
    let input = req.json[User]()?
    let user = db.insert(input)?
    Response.json(user, status: 201)
}`;

const tsCode = `// Zod schema
const UserSchema = z.object({
  name: z.string().min(1).max(100),
  email: z.string().email(),
  age: z.number().int().min(0).max(150),
  role: z.enum(["admin", "member"]).default("member"),
});
// TypeScript type
type User = z.infer<typeof UserSchema>;
// Interface for response
interface UserResponse { ... }

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const input = UserSchema.parse(body);
    const user = await db.insert(input);
    return Response.json(user, { status: 201 });
  } catch (err) {
    if (err instanceof ZodError) {
      return Response.json(
        { error: err.errors }, { status: 400 }
      );
    }
    return Response.json(
      { error: "Internal error" }, { status: 500 }
    );
  }
}`;

function CodeCard({
  code,
  lang,
  filename,
  lines,
  tokens,
  accent,
}: {
  code: string;
  lang: string;
  filename: string;
  lines: number;
  tokens: number;
  accent: string;
}) {
  return (
    <div className="code-block hard-shadow overflow-hidden rounded-xl">
      <div className="flex items-center justify-between border-b border-white/10 px-4 py-2.5">
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-full bg-[#ff5f57]" />
          <div className="h-3 w-3 rounded-full bg-[#febc2e]" />
          <div className="h-3 w-3 rounded-full bg-[#28c840]" />
          <span className="ml-2 text-xs text-white/40">{filename}</span>
        </div>
        <div className="flex items-center gap-3 text-[10px] text-white/30">
          <span>{lines} lines</span>
          <span>~{tokens} tokens</span>
        </div>
      </div>
      <pre className="overflow-x-auto p-4 text-[13px] leading-relaxed">
        <code>{code}</code>
      </pre>
    </div>
  );
}

export function CodeDemo() {
  return (
    <section className="zone-deep relative py-24">
      <div className="section-container">
        <div className="mb-14 text-center">
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="mb-3 font-mono text-xs font-medium uppercase tracking-[0.25em] text-klar-leaf"
          >
            Side by side
          </motion.p>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-3xl font-bold tracking-tight text-white sm:text-4xl"
          >
            Same feature.{" "}
            <span className="text-klar-peach">71% fewer tokens.</span>
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="mx-auto mt-3 max-w-lg text-sm leading-relaxed text-white/50"
          >
            User registration endpoint. Klar auto-generates validation,
            serialization, and error handling from the{" "}
            <span className="font-mono text-klar-mint">@schema</span>{" "}
            annotation.
          </motion.p>
        </div>

        <div className="grid gap-8 lg:grid-cols-2">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <div className="mb-3 flex items-center gap-2">
              <span className="inline-block h-3 w-3 rounded-full bg-klar-leaf" />
              <span className="text-sm font-semibold text-white">Klar</span>
              <span className="ml-auto rounded-full bg-klar-leaf/15 px-2.5 py-0.5 font-mono text-[10px] font-medium text-klar-leaf">
                14 lines
              </span>
            </div>
            <CodeCard
              code={klarCode}
              lang="klar"
              filename="handler.klar"
              lines={14}
              tokens={200}
              accent="var(--klar-leaf)"
            />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <div className="mb-3 flex items-center gap-2">
              <span className="inline-block h-3 w-3 rounded-full bg-klar-sky" />
              <span className="text-sm font-semibold text-white">TypeScript</span>
              <span className="ml-auto rounded-full bg-white/10 px-2.5 py-0.5 font-mono text-[10px] font-medium text-white/50">
                28 lines
              </span>
            </div>
            <CodeCard
              code={tsCode}
              lang="typescript"
              filename="route.ts"
              lines={28}
              tokens={700}
              accent="var(--klar-sky)"
            />
          </motion.div>
        </div>
      </div>
    </section>
  );
}
