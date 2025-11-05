use axum::http::StatusCode;
use sqlx::{postgres::PgRow, PgPool, Row};
use std::collections::HashMap;

pub struct PgsqlService {
    pool: PgPool, // 数据库连接池
}

impl PgsqlService {
    /// 创建新的 PgsqlService 实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 检查指定的 cache_path 是否存在
    pub async fn cache_path_exists(
        &self,
        table_name: &str,
        cache_path: &str,
    ) -> Result<bool, StatusCode> {
        // 校验表名是否合法
        if table_name.is_empty()
            || !table_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 准备查询
        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE cache_path = $1) AS exists",
            table_name
        );

        // 执行查询
        let row = sqlx::query(&query)
            .bind(cache_path)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("查询 cache_path 存在性失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // 获取结果
        let exists: bool = row.try_get("exists").map_err(|e| {
            eprintln!("解析查询结果失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        println!("{} 存在: {}", cache_path, exists);
        Ok(exists)
    }

    // 大表优化的随机获取链接方法（带条件过滤和指定列）
    pub async fn get_random_link(
        &self,
        table_name: &str,
        columns: &[&str],
        conditions: HashMap<&str, &str>,
        limit: Option<u32>, // Added optional limit parameter
    ) -> Result<Vec<PgRow>, StatusCode> {
        // 1. 校验表名和列名安全性
        if !table_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate column names
        for col in columns {
            if !col.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        for key in conditions.keys() {
            if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        // 2. 构建条件查询部分
        let mut where_clause = String::from("WHERE 1=1");
        let mut params = Vec::new();
        for (i, (key, value)) in conditions.iter().enumerate() {
            where_clause.push_str(&format!(" AND {} = ${}", key, i + 1));
            params.push(*value);
        }

        // 3. 使用高效随机算法（避免ORDER BY RANDOM()）
        let column_list = columns.join(", ");

        // Prepare the limit clause
        let limit_clause = limit
            .map(|l| format!("LIMIT {}", l))
            .unwrap_or_else(|| "LIMIT 1".to_string()); // Default to 1 if None

        let query = format!(
            r#"
            WITH random_sample AS (
                SELECT id
                FROM {table_name}
                {where_clause}
                AND id >= (
                    SELECT floor(random() * (
                        SELECT MAX(id) 
                        FROM {table_name}
                        {where_clause}
                    )) + 1
                )
                {limit_clause}
            )
            SELECT t.{columns}
            FROM {table_name} t
            JOIN random_sample rs ON t.id = rs.id
            "#,
            table_name = table_name,
            where_clause = where_clause,
            columns = column_list,
            limit_clause = limit_clause
        );

        // 4. 执行查询
        let mut query = sqlx::query(&query);
        for param in params {
            query = query.bind(param);
        }

        query.fetch_all(&self.pool).await.map_err(|e| {
            eprintln!("大表随机查询失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    // 从表中获取指定列和条件的数据
    pub async fn fetch_data(
        &self,
        table_name: &str,
        columns: &[&str],
        conditions: HashMap<&str, &str>,
        limit: Option<u32>,
        page: Option<u32>,
        per_page: Option<u32>,
        search_term: Option<&str>, // 新增 search_term 参数
    ) -> Result<(Vec<PgRow>, i64), StatusCode> {
        // 校验表名
        if table_name.is_empty()
            || !table_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 参数检查
        if limit.is_some() && (page.is_some() || per_page.is_some()) {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 构建查询
        let mut query_str = format!("SELECT {} FROM {}", columns.join(", "), table_name);
        let mut values: Vec<&str> = Vec::new();

        // 添加条件
        let mut clauses = Vec::new();
        if !conditions.is_empty() {
            clauses = conditions
                .iter()
                .enumerate()
                .map(|(i, (key, value))| {
                    if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    values.push(*value);
                    Ok(format!("{} = ${}", key, i + 1))
                })
                .collect::<Result<_, _>>()?;
        }

        // 添加 search_term 的模糊搜索
        if let Some(term) = search_term {
            let search_pos = values.len() + 1;
            // 假设对 "url" 列进行模糊搜索，你可以根据需要修改为其他列或多列
            clauses.push(format!("url LIKE ${}", search_pos));
            values.push(term);
        }

        if !clauses.is_empty() {
            query_str.push_str(" WHERE ");
            query_str.push_str(&clauses.join(" AND "));
        }

        // 获取总数（仅在分页时计算）
        let total = if page.is_some() || per_page.is_some() {
            let count_query = format!("SELECT COUNT(*) FROM ({}) as subquery", query_str);
            let mut query = sqlx::query_scalar(&count_query);
            for value in &values {
                query = query.bind(*value);
            }
            query.fetch_one(&self.pool).await.map_err(|e| {
                eprintln!("获取总数失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        } else {
            0
        };

        // 添加分页或 LIMIT
        if let Some(limit) = limit {
            query_str.push_str(&format!(" LIMIT ${}", values.len() + 1));
        } else if let (Some(page), Some(per_page)) = (page, per_page) {
            if page == 0 || per_page == 0 || per_page > 1000 {
                return Err(StatusCode::BAD_REQUEST);
            }
            let limit_pos = values.len() + 1;
            let offset_pos = values.len() + 2;
            query_str.push_str(&format!(" LIMIT ${} OFFSET ${}", limit_pos, offset_pos));
        }

        println!("query_str: {:?}", query_str);
        println!("values: {:?}", values);
        // 执行查询
        let mut query = sqlx::query(&query_str);
        for value in &values {
            query = query.bind(*value); // 绑定 WHERE 条件和 search_term 参数
        }

        if let Some(limit) = limit {
            query = query.bind(limit as i64);
        } else if let (Some(page), Some(per_page)) = (page, per_page) {
            query = query.bind(per_page as i64);
            query = query.bind((page - 1) as i64 * per_page as i64);
        }

        let rows = query.fetch_all(&self.pool).await.map_err(|e| {
            eprintln!("获取数据失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok((rows, total))
    }

    // 获取 Website_cache 表数据，如果表不存在则创建
    pub async fn fetch_or_create_table(
        &self,
        table_name: &str,
        columns: &[&str],
        conditions: HashMap<&str, &str>,
        limit: Option<u32>, // 可选限制行数
    ) -> Result<Vec<PgRow>, StatusCode> {
        // 尝试获取数据
        match self
            .fetch_data(
                table_name,
                columns,
                conditions.clone(),
                limit,
                None,
                None,
                None,
            )
            .await
        {
            Ok((rows, _)) => Ok(rows), // 表存在，直接返回数据
            Err(status) => {
                // 如果错误是表不存在（通常是 INTERNAL_SERVER_ERROR，需检查具体错误）
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    // 创建表
                    let create_table_query = format!(
                        r#"
                        CREATE TABLE IF NOT EXISTS {} (
                            id SERIAL PRIMARY KEY,         /* 自增主键 */
                            cache_path TEXT UNIQUE,        /* 唯一缓存路径 */
                            url TEXT,                      /* 访问来路url */
                            uri TEXT,                      /* 真实指向uri(映射时写入) */
                            target TEXT,                   /* 目标内容 */
                            title TEXT,                    /* 页面标题 */
                            keywords TEXT,                 /* 关键词 */
                            description TEXT,              /* 描述 */
                            domain VARCHAR(255),           /* 域名 */
                            root_domain VARCHAR(255),      /* 根域名 */
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 创建时间 */
                            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 更新时间 */
                            page_type VARCHAR(50),         /* 页面类型 缓存/映射/目录/静态 */
                            source TEXT                    /* 来源 */
                        )
                        "#,
                        table_name
                    );
                    sqlx::query(&create_table_query)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            eprintln!("创建表失败: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    // 表创建后，返回空结果（因为新表没有数据）
                    Ok(Vec::new())
                } else {
                    // 其他错误直接返回
                    Err(status)
                }
            }
        }
    }

    pub async fn insert_or_create_website_cache(
        &self,
        table_name: &str,
        data: HashMap<&str, &str>,
    ) -> Result<(), StatusCode> {
        // 校验表名是否合法（只允许字母数字和下划线）
        if table_name.is_empty()
            || !table_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 校验数据中的列名是否合法
        if !data.keys().all(|key| {
            key.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        }) {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 首先尝试直接插入数据
        match self.insert_data(table_name, &data).await {
            Ok(_) => Ok(()), // 插入成功直接返回
            Err(status) => {
                // 如果错误是因为表不存在（这里假设INTERNAL_SERVER_ERROR表示表不存在）
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    // 先创建表
                    let create_table_query = format!(
                        r#"
                        CREATE TABLE IF NOT EXISTS {} (
                            id SERIAL PRIMARY KEY,         /* 自增主键 */
                            cache_path TEXT UNIQUE,        /* 唯一缓存路径 */
                            url TEXT,                      /* 访问来路url */
                            uri TEXT,                      /* 真实指向uri(映射时写入) */
                            target TEXT,                   /* 目标内容 */
                            title TEXT,                    /* 页面标题 */
                            keywords TEXT,                 /* 关键词 */
                            description TEXT,              /* 描述 */
                            domain VARCHAR(255),           /* 域名 */
                            root_domain VARCHAR(255),      /* 根域名 */
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 创建时间 */
                            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 更新时间 */
                            page_type VARCHAR(50),         /* 页面类型 缓存/映射/目录/静态 */
                            source TEXT                    /* 来源 */
                        )
                        "#,
                        table_name
                    );

                    // 执行建表语句
                    sqlx::query(&create_table_query)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            eprintln!("创建表失败: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    // 表创建成功后，再次尝试插入数据
                    self.insert_data(table_name, &data).await
                } else {
                    // 其他错误直接返回
                    Err(status)
                }
            }
        }
    }

    /// 实际执行数据插入的辅助函数
    ///
    /// # 参数
    /// - `table_name`: 表名
    /// - `data`: 要插入的数据
    async fn insert_data(
        &self,
        table_name: &str,
        data: &HashMap<&str, &str>,
    ) -> Result<(), StatusCode> {
        // 检查数据是否为空
        if data.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 准备列名和值
        let columns: Vec<&str> = data.keys().copied().collect();
        let values: Vec<&str> = data.values().copied().collect();

        // 生成占位符 ($1, $2, ...)
        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();

        // 构建INSERT SQL语句
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),      // 列名列表
            placeholders.join(", ")  // 占位符列表
        );

        // 准备查询并绑定参数
        let mut query = sqlx::query(&query);
        for value in values {
            query = query.bind(value);
        }

        // 执行查询
        query
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("插入数据失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|_| ()) // 忽略执行结果，只返回Ok(())
    }

    /// 获取 PostgreSQL 数据库完整版本字符串
    pub async fn get_db_version(&self) -> Result<String, StatusCode> {
        let version_row = self.execute_query("SELECT version()").await?;
        version_row.try_get("version").map_err(|e| {
            eprintln!("获取数据库版本失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    /// 检查 PostgreSQL 主版本号
    /// 返回主版本号（例如 16），并可用于验证是否满足最低版本要求
    pub async fn check_db_version(&self) -> Result<(bool, String), StatusCode> {
        let version_str = self.get_db_version().await?;
        // 提取版本号，例如 "PostgreSQL 16.1 on x86_64-pc-linux-gnu" -> 16
        let version_parts: Vec<&str> = version_str.split_whitespace().collect();
        if version_parts.len() < 2 {
            eprintln!("无法解析版本字符串: {}", version_str);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let version_num = version_parts[1]; // "16.1"
        let major_version = version_num
            .split('.')
            .next()
            .ok_or_else(|| {
                eprintln!("无法提取主版本号: {}", version_num);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .parse::<i32>()
            .map_err(|e| {
                eprintln!("版本号解析失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        if major_version < 16 {
            eprintln!("PostgreSQL 版本{}过低 (低于PGSQL16)", major_version);
            return Ok((false, version_str)); // 版本过低
        }
        Ok((true, version_str)) // 版本满足要求
    }

    /// 执行原始 SQL 查询并返回单行结果
    async fn execute_query(&self, query: &str) -> Result<PgRow, StatusCode> {
        sqlx::query(query).fetch_one(&self.pool).await.map_err(|e| {
            eprintln!("查询执行失败: {}", e); // 记录错误日志
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    /// 获取指定表中的总行数
    pub async fn get_data_count(&self, table_name: &str) -> Result<i64, StatusCode> {
        // 防止 SQL 注入，验证表名
        if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(StatusCode::BAD_REQUEST);
        }

        let query = format!("SELECT COUNT(*) FROM {}", table_name);
        let count_row = self.execute_query(&query).await?;
        count_row.try_get("count").map_err(|e| {
            eprintln!("获取计数失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    pub async fn get_paginated_tables(
        &self,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
        page: i64,     // 改为 i64
        per_page: i64, // 改为 i64
    ) -> Result<(Vec<String>, u64), StatusCode> {
        // 参数校验
        if page <= 0 || per_page <= 0 || per_page > 1000 {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 1. 获取总数
        let (count_query, count_params) = self.build_count_query(starts_with, ends_with);
        let mut query = sqlx::query_scalar(&count_query);

        for param in &count_params {
            query = query.bind(param);
        }

        let total: i64 = query.fetch_one(&self.pool).await.map_err(|e| {
            eprintln!("查询表数量失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // 2. 获取分页数据
        let offset = (page - 1) * per_page;
        let (data_query, data_params) = self.build_data_query(starts_with, ends_with);

        let mut query = sqlx::query(&data_query);

        // 绑定过滤参数
        for param in &data_params {
            query = query.bind(param);
        }

        // 绑定分页参数（直接使用i64）
        let rows = query
            .bind(per_page) // LIMIT
            .bind(offset) // OFFSET
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("查询表名失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // 3. 提取结果
        let tables = rows
            .into_iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();

        Ok((tables, total as u64))
    }

    // 构建计数查询（保持不变）
    fn build_count_query(
        &self,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
    ) -> (String, Vec<String>) {
        let mut query = String::from(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
        );

        let mut params = Vec::new();
        let mut conditions = Vec::new();

        if let Some(prefix) = starts_with {
            conditions.push("table_name LIKE $1 || '%'".to_string());
            params.push(prefix.to_string());
        }

        if let Some(suffix) = ends_with {
            let pos = params.len() + 1;
            conditions.push(format!("table_name LIKE '%' || ${}", pos));
            params.push(suffix.to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        (query, params)
    }

    // 构建数据查询（修改为不处理分页参数）
    fn build_data_query(
        &self,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
    ) -> (String, Vec<String>) {
        let mut query = String::from(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
        );

        let mut params = Vec::new();
        let mut conditions = Vec::new();

        if let Some(prefix) = starts_with {
            conditions.push("table_name LIKE $1 || '%'".to_string());
            params.push(prefix.to_string());
        }

        if let Some(suffix) = ends_with {
            let pos = params.len() + 1;
            conditions.push(format!("table_name LIKE '%' || ${}", pos));
            params.push(suffix.to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        // 添加分页占位符（参数将在主函数中绑定）
        let limit_pos = params.len() + 1;
        let offset_pos = params.len() + 2;
        query.push_str(&format!(
            " ORDER BY table_name LIMIT ${} OFFSET ${}",
            limit_pos, offset_pos
        ));

        (query, params)
    }
}

// // 示例用法
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // 初始化数据库连接池（实际应用中应使用适当的配置）
//     let pool = PgPool::connect("postgres://user:password@localhost:5432/dbname").await?;
//     let db_service = PgsqlService::new(pool);

//     // 示例 1: 获取数据库版本
//     match db_service.get_db_version().await {
//         Ok(version) => println!("数据库版本: {}", version),
//         Err(status) => println!("获取版本失败: {}", status),
//     }

//     // 示例 2: 获取表行数
//     match db_service.get_data_count("users").await {
//         Ok(count) => println!("用户总数: {}", count),
//         Err(status) => println!("获取计数失败: {}", status),
//     }

//     // 示例 3: 获取特定数据
//     let mut conditions = HashMap::new();
//     conditions.insert("age", "25");      // 添加年龄条件
//     conditions.insert("status", "active"); // 添加状态条件

//     let columns = vec!["id", "name", "email"]; // 要查询的列
//     match db_service.fetch_data("users", &columns, conditions).await {
//         Ok(rows) => {match -           for row in rows {
//                 let id: i32 = row.try_get("id")?;
//                 let name: String = row.try_get("name")?;
//                 let email: String = row.try_get("email")?;
//                 println!("用户信息: {} - {} - {}", id, name, email);
//             }
//         }
//         Err(status) => println!("获取数据失败: {}", status),
//     }

//     Ok(())
// }
