#!/bin/bash

cd /www

# 检查是否已经存在 "MirrorElf" 目录
if [ -d "MirrorElf" ]; then
  echo "目录 'MirrorElf' 存在，开始更新"
else
  echo "目录 'MirrorElf' 不存在，请先安装程序。"
  exit 0
  # 在这里添加你需要执行的命令
fi

# 安装 jq 和 tar，如果它们尚未安装
if command -v yum &>/dev/null; then
  echo "CentOS系统"
  yum install -y jq tar
else
  echo "Debian/Ubuntu系统"
  apt install -y jq tar
fi

# 从 GitHub API 获取最新的发布信息
RELEASE_JSON=$(curl -s https://api.github.com/repos/seo888/MirrorElf-R/releases/latest)

# 从 JSON 响应中提取 tarball URL
TAR_URL=$(echo "$RELEASE_JSON" | jq -r .tarball_url)

# 检查 TAR URL 是否为空
if [ -z "$TAR_URL" ]; then
  echo "从 GitHub API 获取 TAR URL 失败"
  exit 1
fi

# 定义基于版本标签的输出 tar 文件名
TAR_FILE="MirrorElf-$(echo "$RELEASE_JSON" | jq -r .tag_name).tar.gz"

TARGET_DIR="MirrorElf_New"

# 使用 curl 下载 tarball
echo "从 $TAR_URL 下载发布版本..."
curl -L -o "$TAR_FILE" "$TAR_URL"

# 检查下载是否成功
if [ $? -eq 0 ]; then
  echo "下载成功！"
else
  echo "下载失败！"
  exit 1
fi

# 检查文件是否存在
# if [ ! -f "$TAR_FILE" ]; then
#   echo "错误：文件 $TAR_FILE 不存在"
#   exit 1
# fi

rm -rf "$TARGET_DIR"

# 创建目标目录
mkdir -p "$TARGET_DIR"

# 尝试不同解压方式
if tar --help | grep -q "one-top-level"; then
  # 支持新语法
  tar -xzf "$TAR_FILE" --one-top-level="$TARGET_DIR" --strip-components=1
else
  # 传统方式
  TEMP_DIR=$(mktemp -d)
  tar -xzf "$TAR_FILE" -C "$TEMP_DIR" --strip-components=1
  mv "$TEMP_DIR"/* "$TARGET_DIR"/
  rm -rf "$TEMP_DIR"
fi

# 检查结果
if [ $? -eq 0 ]; then
  echo "解压成功到 $TARGET_DIR/"
  rm -f "$TAR_FILE"
else
  echo "解压失败！"
  exit 1
fi

echo "'镜像精灵 MirrorElf $(echo "$RELEASE_JSON" | jq -r .tag_name) 下载成功'"

new="/www/MirrorElf_New/app/Mirror-Elf-Rust"
old="/www/MirrorElf/app/Mirror-Elf-Rust"
mv -f "$new" "$old"

new="/www/MirrorElf_New/app/config/IP2LOCATION-LITE-DB3.BIN"
old="/www/MirrorElf/app/config/IP2LOCATION-LITE-DB3.BIN"
mv -f "$new" "$old"

new="/www/MirrorElf_New/app/config/china.json"
old="/www/MirrorElf/app/config/china.json"
mv -f "$new" "$old"

new="/www/MirrorElf_New/docker-compose.yml"
old="/www/MirrorElf/docker-compose.yml"
mv -f "$new" "$old"

new="/www/MirrorElf_New/update.sh"
old="/www/MirrorElf/update.sh"
mv -f "$new" "$old"

# new="/www/MirrorElf_New/app/_"
# old="/www/MirrorElf/app/_"
# # 确保目标目录存在
# if [ -d "$old" ]; then
#     rm -rf "$old"/*  # 删除目标目录中的所有内容
# fi
# # 使用 cp 命令复制新目录的内容到目标目录
# cp -r "$new"/* "$old"/

new="/www/MirrorElf_New/app/_"
old="/www/MirrorElf/app/_"
backup_dir="/tmp/js_backup" # 临时备份目录

# 1. 备份原 js 目录
if [ -d "$old/static/js" ]; then
  mkdir -p "$backup_dir"
  cp -r "$old/static/js" "$backup_dir/"
fi

# 2. 完全清空目标目录
if [ -d "$old" ]; then
  find "$old" -mindepth 1 -delete
fi

# 3. 复制新内容
cp -r "$new"/* "$old/"

# 4. 恢复原 js 目录
if [ -d "$backup_dir/js" ]; then
  mkdir -p "$old/static/"
  cp -r "$backup_dir/js" "$old/static/"
  rm -rf "$backup_dir" # 清理备份
fi

PROJECT_DIR="/www/MirrorElf"
# 切换到项目目录
cd "$PROJECT_DIR" || exit 1

# 检查并创建 postgres_data 目录（如果不存在）
POSTGRES_DATA_DIR="/www/MirrorElf/postgres_data"
if [ ! -d "$POSTGRES_DATA_DIR" ]; then
  echo "创建 PostgreSQL 数据目录: $POSTGRES_DATA_DIR"
  mkdir -p "$POSTGRES_DATA_DIR"
  # 设置权限为 PostgreSQL 用户（UID 999）
  chown 999:999 "$POSTGRES_DATA_DIR"
  chmod 700 "$POSTGRES_DATA_DIR"
fi

# 检查并config.yml
app="/www/MirrorElf/app"

# 定义配置文件路径
config_file="/www/MirrorElf/app/config/config.yml"

# 定义替换文本
replacement_text=$'  external_filter:\n    - .gov.cn\n  external_links:\n    - \'{随机网址}\'\n  meta_information: false\n  random_div_attributes: true\n  random_class_name: false\n  head_header: \'\'\n  head_footer: \'\'\n  body_header: <h1><a target="_blank" title="{标题}" href="{首页}">{核心词}</a></h1>\n  body_footer: <a target="_blank" href="{@push_link}">{@keyword}</a>\n  html_entities: false\n  friend_link_count: 5\n  friend_links:\n    - <a target="_blank" title="{*主站.标题#1001}" href="{*主站.首页#1001}">{*主站.核心词#1001}</a>\n  seo_404_page: false'

# 写入临时文件
printf "%s\n" "$replacement_text" > /tmp/temp_replacement.txt

# # 检查临时文件内容（调试用）
# echo "临时文件内容："
# cat /tmp/temp_replacement.txt

# 检查配置文件是否包含必要标记
if grep -q "^SEOFunctions:" "$config_file" && grep -q "^AccessPolicy:" "$config_file" && ! grep -q "head_footer" "$config_file"; then
  # 替换内容
  sed -i.bak '
    /^SEOFunctions:/,/^AccessPolicy:/ {
      /^SEOFunctions:/ {
        p
        r /tmp/temp_replacement.txt
        d
      }
      /^AccessPolicy:/ {
        p
        d
      }
      d
    }
  ' "$config_file"
  echo "替换完成"
fi

# 清理临时文件
rm -f /tmp/temp_replacement.txt

# 重启容器
docker compose down && docker compose up -d || exit 1
