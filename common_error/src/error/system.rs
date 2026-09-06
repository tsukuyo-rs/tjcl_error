use crate::define_error_detail;

/// システムサブエラー詳細
define_error_detail!(SystemSub {
    /// 未初期化状態でのアクセス
    NotInitialized = 0x01,
    /// 二重初期化
    AlreadyInitialized = 0x02,
    /// 不正なパラメータ
    InvalidParameter = 0x03,
    /// 処理タイムアウト
    Timeout = 0x04,
    /// ハードウェアフォルト
    HardwareFault = 0x05,
});
