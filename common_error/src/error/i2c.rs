use crate::define_error_detail;

/// I2C通信サブエラー詳細
define_error_detail!(I2cSub {
    /// バスビジー（ラインが塞がっている）
    BusBusy = 0x01,
    /// アービトレーションロスト（調停衝突）
    ArbitrationLost = 0x02,
    /// アドレス送信時のNACK応答
    AddressNack = 0x03,
    /// データ送信時のNACK応答
    DataNack = 0x04,
    /// 通信タイムアウト
    Timeout = 0x05,
    /// 受信バッファオーバーラン
    Overrun = 0x06,
});
