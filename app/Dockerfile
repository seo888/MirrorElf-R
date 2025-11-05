# # 使用轻量级的基础镜像
# FROM debian:bullseye-slim

# # 设置工作目录
# WORKDIR /app

# # 将编译好的二进制文件复制到容器中
# COPY . .

# # 暴露端口
# EXPOSE 16888

# # 设置启动命令
# CMD ["./Mirror-Elf-Rust"]

# 使用 rust:1.82-slim-bookworm 作为基础镜像
FROM rust:1.82-slim-bookworm

# 安装编译依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    build-essential \
    cmake \
    perl \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制项目源码
COPY . .

# 编译 Rust 程序
RUN cargo build --release --features vendored

# 暴露端口
EXPOSE 16888

# 运行编译后的二进制
CMD ["./target/release/Mirror-Elf-Rust"]