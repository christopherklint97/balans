use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct VoucherAttachment {
    pub id: String,
    pub voucher_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    #[sqlx(skip)]
    #[serde(skip)]
    pub data: Vec<u8>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentMeta {
    pub id: String,
    pub voucher_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

impl From<VoucherAttachment> for AttachmentMeta {
    fn from(a: VoucherAttachment) -> Self {
        Self {
            id: a.id,
            voucher_id: a.voucher_id,
            filename: a.filename,
            content_type: a.content_type,
            size_bytes: a.size_bytes,
            created_at: a.created_at,
        }
    }
}
