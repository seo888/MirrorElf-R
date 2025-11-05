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
            "SELECT EXISTS(SELECT 1 FROM \"{}\" WHERE cache_path = $1) AS exists",
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
        // println!("{} 存在: {}", cache_path, exists);
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
                FROM "{table_name}"
                {where_clause}
                AND id >= (
                    SELECT floor(random() * (
                        SELECT MAX(id) 
                        FROM "{table_name}"
                        {where_clause}
                    )) + 1
                )
                {limit_clause}
            )
            SELECT t.{columns}
            FROM "{table_name}" t
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
    // pub async fn fetch_data(
    //     &self,
    //     table_name: &str,
    //     columns: &[&str],
    //     conditions: HashMap<&str, &str>,
    //     limit: Option<u32>,
    //     page: Option<u32>,
    //     per_page: Option<u32>,
    //     search_term: Option<&str>,
    //     sort: Option<(&str, &str)>,
    // ) -> Result<(Vec<PgRow>, i64), StatusCode> {
    //     // println!("page:{:?}", page);
    //     // println!("per_page:{:?}", per_page);
    //     // 校验表名
    //     if table_name.is_empty()
    //         || !table_name
    //             .chars()
    //             .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    //     {
    //         return Err(StatusCode::BAD_REQUEST);
    //     }

    //     // 参数检查
    //     if limit.is_some() && (page.is_some() || per_page.is_some()) {
    //         return Err(StatusCode::BAD_REQUEST);
    //     }

    //     // 构建查询
    //     let column_str = if columns.is_empty() {
    //         "*".to_string()
    //     } else {
    //         columns.join(", ")
    //     };
    //     let mut query_str = format!("SELECT {} FROM \"{}\"", column_str, table_name);
    //     let mut values: Vec<&str> = Vec::new();
    //     let mut clauses = Vec::new();

    //     // 添加普通条件
    //     for (key, value) in conditions {
    //         if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
    //             return Err(StatusCode::BAD_REQUEST);
    //         }
    //         let (op, actual_value) = if value.starts_with("!=") {
    //             ("!=", value.trim_start_matches("!=").trim())
    //         } else if value.starts_with("LIKE ") {
    //             ("LIKE", value.trim_start_matches("LIKE ").trim())
    //         } else if value.starts_with("NOT LIKE ") {
    //             ("NOT LIKE", value.trim_start_matches("NOT LIKE ").trim())
    //         } else {
    //             ("=", value)
    //         };
    //         clauses.push(format!("{} {} ${}", key, op, values.len() + 1));
    //         values.push(actual_value);
    //     }

    //     // 添加搜索条件（搜索 domain, title, description）
    //     let mut param_strings: Vec<String> = Vec::new();
    //     if let Some(term) = search_term {
    //         if !term.is_empty() {
    //             if term.contains('\n') {
    //                 let terms: Vec<&str> = term.split('\n').collect();
    //                 let mut domain_clauses: Vec<String> = Vec::new();

    //                 for sub_term in terms.iter() {
    //                     if !sub_term.is_empty() {
    //                         let search_param = format!("%{}%", sub_term);
    //                         domain_clauses.push(format!(
    //                             "domain LIKE ${}",
    //                             values.len() + 1 + param_strings.len()
    //                         ));
    //                         param_strings.push(search_param);
    //                     }
    //                 }

    //                 if !domain_clauses.is_empty() {
    //                     clauses.push(format!("({})", domain_clauses.join(" OR ")));
    //                 }
    //             } else {
    //                 let search_param = format!("%{}%", term);
    //                 clauses.push(format!(
    //                     "domain LIKE ${} OR title LIKE ${} OR keywords LIKE ${} OR description LIKE ${} OR target LIKE ${}",
    //                     values.len() + 1,
    //                     values.len() + 1,
    //                     values.len() + 1,
    //                     values.len() + 1,
    //                     values.len() + 1
    //                 ));
    //                 param_strings.push(search_param);
    //             }

    //             // 存储 &str 引用
    //             for param in param_strings.iter() {
    //                 values.push(param.as_str());
    //             }
    //         }
    //     }

    //     // 组合所有条件
    //     if !clauses.is_empty() {
    //         query_str.push_str(" WHERE ");
    //         query_str.push_str(&clauses.join(" AND "));
    //     }

    //     // 添加排序
    //     if let Some((sort_field, sort_direction)) = sort {
    //         if !sort_field.chars().all(|c| c.is_alphanumeric() || c == '_') {
    //             return Err(StatusCode::BAD_REQUEST);
    //         }
    //         let direction = match sort_direction.to_uppercase().as_str() {
    //             "ASC" => "ASC",
    //             "DESC" => "DESC",
    //             _ => return Err(StatusCode::BAD_REQUEST),
    //         };
    //         query_str.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
    //     } else {
    //         query_str.push_str(" ORDER BY id ASC");
    //     }

    //     // 获取总数（仅在分页时计算）
    //     let total = if page.is_some() || per_page.is_some() {
    //         let count_query = format!("SELECT COUNT(*) FROM ({}) as subquery", query_str);
    //         let mut query = sqlx::query_scalar(&count_query);
    //         for value in &values {
    //             query = query.bind(*value);
    //         }
    //         query.fetch_one(&self.pool).await.map_err(|e| {
    //             eprintln!("获取总数失败: {}", e);
    //             StatusCode::INTERNAL_SERVER_ERROR
    //         })?
    //     } else {
    //         0
    //     };

    //     // 添加分页或 LIMIT
    //     if let Some(limit) = limit {
    //         query_str.push_str(&format!(" LIMIT ${}", values.len() + 1));
    //     } else if let (Some(page), Some(per_page)) = (page, per_page) {
    //         if page == 0 || per_page == 0 || per_page > 1000 {
    //             return Err(StatusCode::BAD_REQUEST);
    //         }
    //         query_str.push_str(&format!(
    //             " LIMIT ${} OFFSET ${}",
    //             values.len() + 1,
    //             values.len() + 2
    //         ));
    //     }
    //     // println!("query_str: {}", query_str);

    //     // 执行查询
    //     let mut query = sqlx::query(&query_str);
    //     // 绑定字符串参数
    //     for value in &values {
    //         query = query.bind(*value);
    //     }
    //     // 绑定分页参数
    //     if let Some(limit) = limit {
    //         query = query.bind(limit as i64); // 直接绑定 i64
    //     } else if let (Some(page), Some(per_page)) = (page, per_page) {
    //         query = query
    //             .bind(per_page as i64) // 绑定 LIMIT
    //             .bind(((page - 1) * per_page) as i64); // 绑定 OFFSET
    //     }

    //     let rows = query.fetch_all(&self.pool).await.map_err(|e| {
    //         eprintln!("获取数据失败: {}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;

    //     // println!("{:?} {}", rows, total);

    //     Ok((rows, total))
    // }
    pub async fn fetch_data(
    &self,
    table_name: &str,
    columns: &[&str],
    conditions: HashMap<&str, &str>,
    limit: Option<u32>,
    page: Option<u32>,
    per_page: Option<u32>,
    search_term: Option<&str>,
    sort: Option<(&str, &str)>,
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
    let column_str = if columns.is_empty() {
        "*".to_string()
    } else {
        columns.join(", ")
    };
    let mut query_str = format!("SELECT {} FROM \"{}\"", column_str, table_name);
    let mut values: Vec<Box<dyn std::any::Any + Send + Sync>> = Vec::new();
    let mut clauses = Vec::new();

    // 添加普通条件
    for (key, value) in conditions {
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(StatusCode::BAD_REQUEST);
        }
        let (op, actual_value) = if value.starts_with("!=") {
            ("!=", value.trim_start_matches("!=").trim())
        } else if value.starts_with("LIKE ") {
            ("LIKE", value.trim_start_matches("LIKE ").trim())
        } else if value.starts_with("NOT LIKE ") {
            ("NOT LIKE", value.trim_start_matches("NOT LIKE ").trim())
        } else {
            ("=", value)
        };
        
        clauses.push(format!("{} {} ${}", key, op, values.len() + 1));
        
        // 特别处理 id 字段
        if key == "id" {
            match actual_value.parse::<i32>() {
                Ok(int_val) => values.push(Box::new(int_val)),
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            }
        } else {
            values.push(Box::new(actual_value.to_string()));
        }
    }

    // 添加搜索条件（搜索 domain, title, description）
    let mut param_strings: Vec<String> = Vec::new();
    if let Some(term) = search_term {
        if !term.is_empty() {
            if term.contains('\n') {
                let terms: Vec<&str> = term.split('\n').collect();
                let mut domain_clauses: Vec<String> = Vec::new();

                for sub_term in terms.iter() {
                    if !sub_term.is_empty() {
                        let search_param = format!("%{}%", sub_term);
                        domain_clauses.push(format!(
                            "domain LIKE ${}",
                            values.len() + 1 + param_strings.len()
                        ));
                        param_strings.push(search_param);
                    }
                }

                if !domain_clauses.is_empty() {
                    clauses.push(format!("({})", domain_clauses.join(" OR ")));
                }
            } else {
                let search_param = format!("%{}%", term);
                clauses.push(format!(
                    "domain LIKE ${} OR title LIKE ${} OR keywords LIKE ${} OR description LIKE ${} OR target LIKE ${}",
                    values.len() + 1,
                    values.len() + 1,
                    values.len() + 1,
                    values.len() + 1,
                    values.len() + 1
                ));
                param_strings.push(search_param);
            }

            // 存储参数
            for param in param_strings {
                values.push(Box::new(param));
            }
        }
    }

    // 组合所有条件
    if !clauses.is_empty() {
        query_str.push_str(" WHERE ");
        query_str.push_str(&clauses.join(" AND "));
    }

    // 添加排序
    if let Some((sort_field, sort_direction)) = sort {
        if !sort_field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(StatusCode::BAD_REQUEST);
        }
        let direction = match sort_direction.to_uppercase().as_str() {
            "ASC" => "ASC",
            "DESC" => "DESC",
            _ => return Err(StatusCode::BAD_REQUEST),
        };
        query_str.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
    } else {
        query_str.push_str(" ORDER BY id ASC");
    }

    // 获取总数（仅在分页时计算）
    let total = if page.is_some() || per_page.is_some() {
        let count_query = format!("SELECT COUNT(*) FROM ({}) as subquery", query_str);
        let mut query = sqlx::query_scalar(&count_query);
        
        // 绑定参数
        for value in &values {
            if let Some(int_val) = value.downcast_ref::<i32>() {
                query = query.bind(int_val);
            } else if let Some(str_val) = value.downcast_ref::<String>() {
                query = query.bind(str_val);
            }
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
        query_str.push_str(&format!(
            " LIMIT ${} OFFSET ${}",
            values.len() + 1,
            values.len() + 2
        ));
    }

    // 执行查询
    let mut query = sqlx::query(&query_str);
    
    // 绑定参数
    for value in &values {
        if let Some(int_val) = value.downcast_ref::<i32>() {
            query = query.bind(int_val);
        } else if let Some(str_val) = value.downcast_ref::<String>() {
            query = query.bind(str_val);
        }
    }
    
    // 绑定分页参数
    if let Some(limit) = limit {
        query = query.bind(limit as i64);
    } else if let (Some(page), Some(per_page)) = (page, per_page) {
        query = query
            .bind(per_page as i64)
            .bind(((page - 1) * per_page) as i64);
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
                None,
            )
            .await
        {
            Ok((rows, _)) => Ok(rows), // 表存在，直接返回数据
            Err(status) => {
                // 如果错误是表不存在（通常是 INTERNAL_SERVER_ERROR，需检查具体错误）
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    // 创建表
                    // let create_table_query = format!(
                    //     r#"
                    //     CREATE TABLE IF NOT EXISTS "{}" (
                    //         id SERIAL PRIMARY KEY,         /* 自增主键 */
                    //         cache_path TEXT UNIQUE,        /* 唯一缓存路径 */
                    //         url TEXT,                      /* 访问来路url */
                    //         uri TEXT,                      /* 真实指向uri(映射时写入) */
                    //         target TEXT,                   /* 目标内容 */
                    //         title TEXT,                    /* 页面标题 */
                    //         keywords TEXT,                 /* 关键词 */
                    //         description TEXT,              /* 描述 */
                    //         domain VARCHAR(255),           /* 域名 */
                    //         root_domain VARCHAR(255),      /* 根域名 */
                    //         created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 创建时间 */
                    //         updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,  /* 更新时间 */
                    //         page_type VARCHAR(50),         /* 页面类型 缓存/映射/目录/静态 */
                    //         source TEXT                    /* 来源 */
                    //     )
                    //     "#,
                    //     table_name
                    // );
                    // sqlx::query(&create_table_query)
                    //     .execute(&self.pool)
                    //     .await
                    //     .map_err(|e| {
                    //         eprintln!("创建表失败: {}", e);
                    //         StatusCode::INTERNAL_SERVER_ERROR
                    //     })?;

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
        overwrite: bool,
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
        match self.insert_data(table_name, &data, overwrite).await {
            Ok(_) => Ok(()), // 插入成功直接返回
            Err(status) => {
                // 如果错误是因为表不存在（这里假设INTERNAL_SERVER_ERROR表示表不存在）
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    // 先创建表
                    let create_table_query = format!(
                        r#"
                        CREATE TABLE IF NOT EXISTS "{}" (
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
                    self.insert_data(table_name, &data, overwrite).await
                } else {
                    // 其他错误直接返回
                    Err(status)
                }
            }
        }
    }

    pub async fn insert_or_create_config(
        &self,
        table_name: &str,
        data: HashMap<&str, &str>,
        overwrite: bool,
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
        };

        // 首先尝试直接插入数据
        match self.insert_data(table_name, &data, overwrite).await {
            Ok(_) => Ok(()), // 插入成功直接返回
            Err(status) => {
                // 如果错误是因为表不存在（这里假设INTERNAL_SERVER_ERROR表示表不存在）
                if status == StatusCode::INTERNAL_SERVER_ERROR {
                    // 1. 创建表的 SQL
                    let create_table_query = format!(
                        r#"
                        CREATE TABLE IF NOT EXISTS "{}" (
                            id SERIAL PRIMARY KEY,
                            domain VARCHAR(255) UNIQUE,
                            subdomain VARCHAR(255),
                            root_domain VARCHAR(255),
                            target VARCHAR(255),
                            to_lang VARCHAR(50),
                            title TEXT,
                            keywords TEXT,
                            description TEXT,
                            link_mapping BOOLEAN DEFAULT FALSE,
                            replace_mode INTEGER DEFAULT 0,
                            replace_rules_all TEXT[],
                            replace_rules_index TEXT[],
                            replace_rules_page TEXT[],
                            mulu_tem_max INTEGER DEFAULT 0,
                            mulu_mode TEXT,
                            mulu_static BOOLEAN DEFAULT FALSE,
                            mulu_template TEXT[],
                            mulu_custom_header TEXT[],
                            mulu_keywords_file TEXT[],
                            homepage_update_time INTEGER DEFAULT 0,
                            google_include_info TEXT[],
                            bing_include_info TEXT[],
                            baidu_include_info TEXT[],
                            sogou_include_info TEXT[],
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                        );
                        "#,
                        table_name
                    );

                    // 执行创建表语句
                    sqlx::query(&create_table_query)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            eprintln!("创建表失败: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    // 2. 创建更新时间触发器的函数
                    let create_function_query = r#"
                        CREATE OR REPLACE FUNCTION update_updated_at_column()
                        RETURNS TRIGGER AS $$
                        BEGIN
                            NEW.updated_at = CURRENT_TIMESTAMP;
                            RETURN NEW;
                        END;
                        $$ LANGUAGE plpgsql;
                    "#;

                    // 执行创建函数语句
                    sqlx::query(create_function_query)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            eprintln!("创建触发器函数失败: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    // 3. 创建触发器
                    let create_trigger_query = format!(
                        r#"
                        CREATE TRIGGER trigger_update_updated_at
                        BEFORE UPDATE ON "{}"
                        FOR EACH ROW
                        EXECUTE FUNCTION update_updated_at_column();
                        "#,
                        table_name
                    );

                    // 执行创建触发器语句
                    sqlx::query(&create_trigger_query)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            eprintln!("创建触发器失败: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;

                    // 表和触发器创建成功后，再次尝试插入数据
                    self.insert_data(table_name, &data, overwrite).await
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
        overwrite: bool,
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

        // 收集 map 结果为 Vec<String>
        let column_names: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
        let set_clause: Vec<String> = columns
            .iter()
            .map(|c| format!("\"{}\" = EXCLUDED.\"{}\"", c, c))
            .collect();

        let query;
        if table_name == "website_config" {
            // 构建 INSERT SQL 语句
            query = if overwrite {
                format!(
                    r#"INSERT INTO "{}" ({}) VALUES ({}) ON CONFLICT ("domain") DO UPDATE SET {}"#,
                    table_name,
                    column_names.join(", "), // 使用 Vec<String> 的 join
                    placeholders.join(", "),
                    set_clause.join(", ") // 使用 Vec<String> 的 join
                )
            } else {
                format!(
                    r#"INSERT INTO "{}" ({}) VALUES ({}) ON CONFLICT ("domain") DO NOTHING"#,
                    table_name,
                    column_names.join(", "), // 使用 Vec<String> 的 join
                    placeholders.join(", ")
                )
            };
        } else {
            query = format!(
                r#"INSERT INTO "{}" ({}) VALUES ({})"#,
                table_name,
                column_names.join(", "),
                placeholders.join(", ")
            );
        }

        // 准备查询并绑定参数
        let mut query = sqlx::query(&query);
        for (i, column) in columns.iter().enumerate() {
            let value = values[i];
            match *column {
                // 处理布尔字段
                "link_mapping" | "mulu_static" => match value.to_lowercase().as_str() {
                    "true" => query = query.bind(true),
                    "false" => query = query.bind(false),
                    "" => query = query.bind(None::<bool>),
                    _ => {
                        eprintln!("无效的布尔值 for {}: {}", column, value);
                        return Err(StatusCode::BAD_REQUEST);
                    }
                },
                // 处理整数字段
                "replace_mode" | "mulu_tem_max" | "homepage_update_time" => {
                    if value.is_empty() {
                        query = query.bind(None::<i32>);
                    } else {
                        let num: i32 = value.parse().map_err(|_| {
                            eprintln!("无效的整数值 for {}: {}", column, value);
                            StatusCode::BAD_REQUEST
                        })?;
                        query = query.bind(num);
                    }
                }
                // 处理 TEXT[] 字段
                "replace_rules_all"
                | "replace_rules_index"
                | "replace_rules_page"
                | "mulu_template"
                | "mulu_custom_header"
                | "mulu_keywords_file"
                | "google_include_info"
                | "bing_include_info"
                | "baidu_include_info"
                | "sogou_include_info" => {
                    if value == "{}" || value.is_empty() {
                        query = query.bind(Vec::<String>::new());
                    } else {
                        let array: Vec<String> = value
                            .trim_start_matches('{')
                            .trim_end_matches('}')
                            .split(',')
                            .map(|s| {
                                let s = s.trim();
                                if s.starts_with('"') && s.ends_with('"') {
                                    s[1..s.len() - 1].replace("\\\"", "\"").to_string()
                                } else {
                                    s.to_string()
                                }
                            })
                            .collect();
                        query = query.bind(array);
                    }
                }
                // 其他字段（包括 domain）作为字符串绑定
                _ => {
                    query = query.bind(value);
                }
            }
        }

        // 执行查询
        let result = query.execute(&self.pool).await.map_err(|e| {
            eprintln!("插入数据失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // 如果 overwrite = false 且没有插入记录，返回 CONFLICT
        if !overwrite && result.rows_affected() == 0 {
            return Err(StatusCode::CONFLICT);
        }

        Ok(())
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

        let query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
        let count_row = self.execute_query(&query).await?;
        count_row.try_get("count").map_err(|e| {
            eprintln!("获取计数失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    fn build_query(
        &self,
        query_type: &str,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
    ) -> (String, Vec<String>) {
        let mut params = Vec::new();
        let mut conditions = Vec::new();

        // // 基础查询
        // let base_query = if query_type == "count" {
        //     "SELECT COUNT(*) FROM pg_class WHERE relkind = 'r' AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public')"
        // } else {
        //     "SELECT relname AS table_name FROM pg_class WHERE relkind = 'r' AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public')"
        // };
        let base_query = if query_type == "count" {
            "SELECT COUNT(*) FROM pg_class WHERE relkind = 'r' AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public') AND relname != 'website_config'"
        } else {
            "SELECT relname AS table_name FROM pg_class WHERE relkind = 'r' AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public') AND relname != 'website_config'"
        };

        // 处理 starts_with，使用正则表达式精确匹配
        if let Some(prefix) = starts_with {
            let regex_pattern = format!("^{}__[a-zA-Z0-9_]+", regex::escape(prefix));
            conditions.push("relname ~ $1".to_string());
            params.push(regex_pattern);
        }

        // 处理 ends_with
        if let Some(suffix) = ends_with {
            let pos = params.len() + 1;
            conditions.push(format!("relname LIKE '%' || ${}", pos));
            params.push(suffix.replace(".", "_"));
        }

        // 构建完整查询
        let mut query = base_query.to_string();
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }

        // 如果是 data 查询，添加 ORDER BY, LIMIT, OFFSET
        if query_type == "data" {
            let limit_pos = params.len() + 1;
            let offset_pos = params.len() + 2;
            query.push_str(&format!(
                " ORDER BY relname LIMIT ${} OFFSET ${}",
                limit_pos, offset_pos
            ));
        }

        (query, params)
    }

    pub async fn get_paginated_tables(
        &self,
        starts_with: Option<&str>,
        ends_with: Option<&str>,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<(i64, String)>, u64), StatusCode> {
        // 参数校验
        if page <= 0 || per_page <= 0 || per_page > 1000 {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 1. 获取总数
        let (count_query, count_params) = self.build_query("count", starts_with, ends_with);
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
        let (data_query, data_params) = self.build_query("data", starts_with, ends_with);
        let mut query = sqlx::query(&data_query);
        for param in &data_params {
            query = query.bind(param);
        }
        query = query.bind(per_page).bind(offset); // 绑定 LIMIT 和 OFFSET

        let rows = query.fetch_all(&self.pool).await.map_err(|e| {
            eprintln!("查询表名失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // 3. 提取结果
        let tables = rows
            .into_iter()
            .enumerate()
            .map(|(idx, row)| {
                let id = offset + idx as i64 + 1;
                (id, row.get::<String, _>("table_name"))
            })
            .collect();

        Ok((tables, total as u64))
    }

    // 删除指定表（最快方法）
    pub async fn drop_table(
        &self,
        table_name: &str,
        cascade: bool, // 是否强制删除依赖
    ) -> Result<(), StatusCode> {
        // 校验表名
        if table_name.is_empty()
            || !table_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 构建 DROP TABLE 查询
        let cascade_option = if cascade { " CASCADE" } else { "" };
        let query = format!("DROP TABLE IF EXISTS \"{}\"{}", table_name, cascade_option);

        // 执行删除
        sqlx::query(&query).execute(&self.pool).await.map_err(|e| {
            eprintln!("删除表 {} 失败: {}", table_name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // println!("表 {} 已成功删除", table_name);
        Ok(())
    }

    // 根据条件删除表中的数据
    pub async fn delete_data(
        &self,
        table_name: &str,
        conditions: HashMap<&str, &str>,
    ) -> Result<u64, StatusCode> {
        // 校验表名
        if table_name.is_empty()
            || !table_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 校验条件中的键名
        if !conditions
            .keys()
            .all(|key| key.chars().all(|c| c.is_alphanumeric() || c == '_'))
        {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 拒绝无条件的删除操作
        if conditions.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 对键进行排序以确保参数顺序一致
        let mut sorted_keys: Vec<_> = conditions.keys().collect();
        sorted_keys.sort_unstable();

        // 预校验所有值（特别是id需要是整数）
        for &key in &sorted_keys {
            let value = conditions.get(key).unwrap();
            if key == &"id" && value.parse::<i32>().is_err() {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        // 构建WHERE子句
        let where_parts: Vec<String> = sorted_keys
            .iter()
            .enumerate()
            .map(|(i, &&key)| format!("{} = ${}", key, i + 1))
            .collect();

        // 构建完整SQL语句
        // let query_str = format!("DELETE FROM {} WHERE {}", table_name, where_parts.join(" AND "));
        let query_str = format!(
            "DELETE FROM \"{}\" WHERE {}",
            table_name,
            where_parts.join(" AND ")
        );

        // 准备查询并绑定参数
        let mut query = sqlx::query(&query_str);
        for &&key in &sorted_keys {
            let value = conditions.get(key).unwrap();
            if key == "id" {
                query = query.bind(value.parse::<i32>().unwrap());
            } else {
                query = query.bind(*value);
            }
        }

        // 执行查询
        let rows_affected = query
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("删除数据失败: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .rows_affected();

        println!("从表 {} 删除 {} 行数据", table_name, rows_affected);
        Ok(rows_affected)
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
