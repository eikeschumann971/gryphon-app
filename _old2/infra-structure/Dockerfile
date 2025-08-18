FROM rust:1.72

WORKDIR /app

COPY ./infra-structure/simulator /app/simulator
COPY ./infra-structure/path_planner /app/path_planner

WORKDIR /app/simulator
RUN cargo build --release

WORKDIR /app/path_planner
RUN cargo build --release

CMD ["sleep", "infinity"]
