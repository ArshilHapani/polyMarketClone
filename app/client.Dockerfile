FROM oven/bun:slim AS builder

WORKDIR /app

COPY package.json bun.lock ./

RUN bun install --frozen-lockfile

COPY . .

RUN bun run build


FROM oven/bun:slim AS base

WORKDIR /app

COPY --from=builder /app /app

EXPOSE 3000

CMD ["bun", "start"]