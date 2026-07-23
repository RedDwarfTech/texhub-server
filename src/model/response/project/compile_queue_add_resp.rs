use crate::model::diesel::tex::custom_tex_models::TexCompQueue;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[allow(non_snake_case)]
pub struct CompileQueueAddResp {
    #[serde(flatten)]
    pub queue: TexCompQueue,
    /// 近 3 次成功编译耗时的最大值（毫秒），无历史数据时为 0
    pub estimated_compile_time: i64,
}

impl CompileQueueAddResp {
    pub fn from_queue(queue: TexCompQueue, estimated_compile_time: i64) -> Self {
        Self {
            queue,
            estimated_compile_time,
        }
    }
}
