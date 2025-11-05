# 使用轻量级的基础镜像
FROM debian:bullseye-slim

# 设置工作目录
WORKDIR /app

# 将编译好的二进制文件复制到容器中
COPY . .

# 暴露端口
EXPOSE 16888

# 设置启动命令
CMD ["./Mirror-Elf-Rust"]