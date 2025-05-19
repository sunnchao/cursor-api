/// 动态配置的 API KEY
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyConfig {
    /// 认证令牌（必需）
    #[prost(message, optional, tag = "1")]
    pub auth_token: Option<key_config::TokenInfo>,
    /// 是否禁用图片处理能力
    #[prost(bool, optional, tag = "4")]
    pub disable_vision: Option<bool>,
    /// 是否启用慢速池
    #[prost(bool, optional, tag = "5")]
    pub enable_slow_pool: Option<bool>,
    /// 使用量检查模型规则
    #[prost(message, optional, tag = "6")]
    pub usage_check_models: Option<key_config::UsageCheckModel>,
    /// 包含网络引用
    #[prost(bool, optional, tag = "7")]
    pub include_web_references: Option<bool>,
}
/// Nested message and enum types in `KeyConfig`.
pub mod key_config {
    /// 认证令牌信息
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct TokenInfo {
        /// 用户标识符
        #[prost(string, tag = "1")]
        pub sub: String,
        /// 生成时间（Unix 时间戳）
        #[prost(int64, tag = "2")]
        pub start: i64,
        /// 过期时间（Unix 时间戳）
        #[prost(int64, tag = "3")]
        pub end: i64,
        /// 随机字符串
        #[prost(string, tag = "4")]
        pub randomness: String,
        /// 签名
        #[prost(string, tag = "5")]
        pub signature: String,
        /// 机器ID的SHA256哈希值
        #[prost(bytes = "vec", tag = "6")]
        pub machine_id: Vec<u8>,
        /// MAC地址的SHA256哈希值
        #[prost(bytes = "vec", tag = "7")]
        pub mac_id: Vec<u8>,
        /// 代理名称
        #[prost(string, optional, tag = "8")]
        pub proxy_name: Option<String>,
    }
    /// 使用量检查模型规则
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UsageCheckModel {
        /// 检查类型
        #[prost(enumeration = "usage_check_model::Type", tag = "1")]
        pub r#type: i32,
        /// 模型 ID 列表，当 type 为 TYPE_CUSTOM 时生效
        #[prost(string, repeated, tag = "2")]
        pub model_ids: Vec<String>,
    }
    /// Nested message and enum types in `UsageCheckModel`.
    pub mod usage_check_model {
        /// 检查类型
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum Type {
            /// 未指定
            Default = 0,
            /// 禁用
            Disabled = 1,
            /// 全部
            All = 2,
            /// 自定义列表
            Custom = 3,
        }
        impl Type {
            /// String value of the enum field names used in the ProtoBuf definition.
            ///
            /// The values are not transformed in any way and thus are considered stable
            /// (if the ProtoBuf definition does not change) and safe for programmatic use.
            pub fn as_str_name(&self) -> &'static str {
                match self {
                    Self::Default => "TYPE_DEFAULT",
                    Self::Disabled => "TYPE_DISABLED",
                    Self::All => "TYPE_ALL",
                    Self::Custom => "TYPE_CUSTOM",
                }
            }
            /// Creates an enum from field names used in the ProtoBuf definition.
            pub fn from_str_name(value: &str) -> Option<Self> {
                match value {
                    "TYPE_DEFAULT" => Some(Self::Default),
                    "TYPE_DISABLED" => Some(Self::Disabled),
                    "TYPE_ALL" => Some(Self::All),
                    "TYPE_CUSTOM" => Some(Self::Custom),
                    _ => None,
                }
            }
        }
    }
}
