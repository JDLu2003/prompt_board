use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Prompt {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: String,
}

#[derive(Clone, Debug)]
pub struct PromptDraft {
    pub title: String,
    pub content: String,
    pub tags: String,
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
            "SELECT id, title, content, COALESCE(tags, '')
             FROM prompts
             WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1
             ORDER BY last_used_at DESC, updated_at DESC
             LIMIT 80",
        )?;

        let rows = stmt.query_map(params![like], prompt_from_row)?;
        collect_rows(rows)
    }

    pub fn mark_used(&self, id: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE prompts
             SET last_used_at = CURRENT_TIMESTAMP, updated_at = updated_at
             WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn create(&self, draft: &PromptDraft) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO prompts(title, content, tags, last_used_at, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![draft.title.trim(), draft.content.trim(), draft.tags.trim()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update(&self, id: i64, draft: &PromptDraft) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE prompts
             SET title = ?1, content = ?2, tags = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?4",
            params![
                draft.title.trim(),
                draft.content.trim(),
                draft.tags.trim(),
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn all(&self) -> Result<Vec<Prompt>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, COALESCE(tags, '')
             FROM prompts
             ORDER BY last_used_at DESC, updated_at DESC
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
                last_used_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )?;

        self.add_column_if_missing("last_used_at", "DATETIME")?;
        self.add_column_if_missing("updated_at", "DATETIME")?;

        self.conn.execute_batch(
            "UPDATE prompts
             SET last_used_at = COALESCE(last_used_at, created_at, CURRENT_TIMESTAMP),
                 updated_at = COALESCE(updated_at, created_at, CURRENT_TIMESTAMP);
             CREATE INDEX IF NOT EXISTS idx_prompts_last_used_at ON prompts(last_used_at DESC);
             CREATE INDEX IF NOT EXISTS idx_prompts_updated_at ON prompts(updated_at DESC);",
        )?;
        Ok(())
    }

    fn add_column_if_missing(&self, name: &str, definition: &str) -> Result<(), StoreError> {
        let exists = self.column_exists(name)?;
        if !exists {
            self.conn.execute_batch(&format!(
                "ALTER TABLE prompts ADD COLUMN {name} {definition};"
            ))?;
        }
        Ok(())
    }

    fn column_exists(&self, name: &str) -> Result<bool, StoreError> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(prompts)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for column in columns {
            if column? == name {
                return Ok(true);
            }
        }
        Ok(false)
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
                "请作为资深工程师审查以下代码，重点关注 **bug**、边界条件、可维护性和测试缺口。\n\n请按严重程度排序输出：\n\n```rust\n[代码]\n```",
                "代码, 审查",
            ),
            (
                "需求拆解",
                "请把以下需求拆解成：\n\n- MVP 范围\n- 后续迭代\n- 风险点\n- 验收标准\n\n需求：\n\n[需求]",
                "产品, 规划",
            ),
            (
                "中文邮件",
                "请写一封简洁、友好、专业的中文邮件。\n\n- 收件人：[收件人|同事]\n- 主题：[主题]\n- 语气：[语气|专业友好]\n\n邮件结尾需要包含明确的下一步行动。",
                "邮件, 中文",
            ),
        ];

        for (title, content, tags) in samples {
            self.create(&PromptDraft {
                title: title.to_owned(),
                content: content.to_owned(),
                tags: tags.to_owned(),
            })?;
        }

        Ok(())
    }

    fn migrate_seed_examples(&self) -> Result<(), StoreError> {
        let has_english_mail = self
            .conn
            .query_row(
                "SELECT id FROM prompts
                 WHERE title = ?1 AND content LIKE 'Write a concise and friendly email%'
                 LIMIT 1",
                params!["英文邮件"],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        if let Some(id) = has_english_mail {
            self.update(
                id,
                &PromptDraft {
                    title: "中文邮件".to_owned(),
                    content: "请写一封简洁、友好、专业的中文邮件。\n\n- 收件人：[收件人|同事]\n- 主题：[主题]\n- 语气：[语气|专业友好]\n\n邮件结尾需要包含明确的下一步行动。"
                        .to_owned(),
                    tags: "邮件, 中文".to_owned(),
                },
            )?;
        }
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
