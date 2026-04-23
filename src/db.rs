use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Prompt {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: String,
    pub usage_count: i64,
}

pub struct PromptStore {
    conn: Connection,
}

impl PromptStore {
    pub fn open_default() -> Result<Self, StoreError> {
        let db_path = data_dir()?.join("data.db");
        let conn = Connection::open(db_path)?;
        let store = Self { conn };
        store.migrate()?;
        store.migrate_seed_examples()?;
        store.seed_if_empty()?;
        Ok(store)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Prompt>, StoreError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.all();
        }

        let like = format!("%{}%", trimmed);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, COALESCE(tags, ''), usage_count
             FROM prompts
             WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1
             ORDER BY usage_count DESC, created_at DESC
             LIMIT 80",
        )?;

        let rows = stmt.query_map(params![like], prompt_from_row)?;
        collect_rows(rows)
    }

    pub fn increment_usage(&self, id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE prompts SET usage_count = usage_count + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn all(&self) -> Result<Vec<Prompt>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, COALESCE(tags, ''), usage_count
             FROM prompts
             ORDER BY usage_count DESC, created_at DESC
             LIMIT 80",
        )?;

        let rows = stmt.query_map([], prompt_from_row)?;
        collect_rows(rows)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                usage_count INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_prompts_usage ON prompts(usage_count DESC);
            CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at DESC);",
        )?;
        Ok(())
    }

    fn seed_if_empty(&self) -> Result<(), StoreError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))?;

        if count > 0 {
            return Ok(());
        }

        let samples = [
            (
                "改写为更清晰的表达",
                "请将下面这段文字改写得更清晰、自然、专业，保留原意，不要添加不存在的信息：\n\n[原文]",
                "写作, 润色, 中文",
            ),
            (
                "代码审查",
                "请作为资深工程师审查以下代码，重点关注 bug、边界条件、可维护性和测试缺口。请按严重程度排序输出：\n\n[代码]",
                "代码, 审查",
            ),
            (
                "需求拆解",
                "请把以下需求拆解成 MVP 范围、后续迭代、风险点和验收标准：\n\n[需求]",
                "产品, 规划",
            ),
            (
                "中文邮件",
                "请写一封简洁、友好、专业的中文邮件，收件人是[收件人]，主题是[主题]，语气为[语气]。邮件结尾需要包含明确的下一步行动。",
                "邮件, 中文",
            ),
        ];

        for (title, content, tags) in samples {
            self.conn.execute(
                "INSERT INTO prompts(title, content, tags) VALUES (?1, ?2, ?3)",
                params![title, content, tags],
            )?;
        }

        Ok(())
    }

    fn migrate_seed_examples(&self) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE prompts
             SET title = ?1, content = ?2, tags = ?3
             WHERE title = ?4 AND content LIKE 'Write a concise and friendly email%'",
            params![
                "中文邮件",
                "请写一封简洁、友好、专业的中文邮件，收件人是[收件人]，主题是[主题]，语气为[语气]。邮件结尾需要包含明确的下一步行动。",
                "邮件, 中文",
                "英文邮件"
            ],
        )?;
        Ok(())
    }
}

fn data_dir() -> Result<PathBuf, StoreError> {
    let base = dirs::data_dir().ok_or(StoreError::MissingDataDir)?;
    let dir = base.join("Prompt Board");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn prompt_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        tags: row.get(3)?,
        usage_count: row.get(4)?,
    })
}

fn collect_rows<I>(rows: I) -> Result<Vec<Prompt>, StoreError>
where
    I: Iterator<Item = rusqlite::Result<Prompt>>,
{
    let mut prompts = Vec::new();
    for row in rows {
        prompts.push(row?);
    }
    Ok(prompts)
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("could not find a user data directory")]
    MissingDataDir,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}
