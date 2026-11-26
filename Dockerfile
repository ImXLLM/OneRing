# ═══════════════════════════════════════════════════════════════════════════
#  MAIRON — Railway production image (kernel + migrated seed DB + volume boot)
#  Build: docker build -t kian-v2 .
#  Run:   docker run -p 8088:8088 -v kian_data:/data kian-v2
#  Railway: attach a Volume at /data — start-railway.sh seeds /data/kian.db
#           from the pre-migrated community database on first boot.
# ═══════════════════════════════════════════════════════════════════════════
FROM rust:1.100-alpine AS build
WORKDIR /kian
RUN apk add --no-cache musl-dev pkgconfig
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p kian-server

FROM alpine:3.21
RUN apk add --no-cache ca-certificates && addgroup -S kian && adduser -S kian -G kian
WORKDIR /app
COPY --from=build /kian/target/release/kian-server /app/kian-server
COPY seed/kian-production.db /app/seed/kian-production.db
COPY start-railway.sh /app/start-railway.sh
# Sovereign runtime alignment — standalone dashboards + genome/brain assets ship
# inside the image so GET /war-room and GET /neural-hud work live on Railway.
COPY tools/war-room/war_room.html /app/tools/war-room/war_room.html
COPY tools/neural-hud/neural_hud.html /app/tools/neural-hud/neural_hud.html
COPY OMNI-MODEL-GENOME /app/OMNI-MODEL-GENOME
COPY KIA-BRAIN-OS /app/KIA-BRAIN-OS
RUN chmod +x /app/start-railway.sh && mkdir -p /data && chown -R kian:kian /app /data
USER kian
ENV KIAN_HOST=0.0.0.0 \
    KIAN_PORT=8088 \
    KIAN_DATABASE_URL=file:/data/kian.db
EXPOSE 8088
ENTRYPOINT ["/app/start-railway.sh"]
