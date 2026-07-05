FROM oven/bun:1 AS build
WORKDIR /app

COPY package.json bunfig.toml ./
RUN bun install

COPY . .
RUN bun run build

FROM caddy:2-alpine
COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=build /app/dist /srv

EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]