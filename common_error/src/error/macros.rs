#[macro_export]
macro_rules! define_error_kind {
    // $variant: バリアント名
    // $val: 親ID (u8リテラル)
    // $sub_type: 子Enumの型パス
    ( $( $(#[$meta:meta])* $variant:ident = $val:literal => $sub_type:path ), *$(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum Kind {
            $( $(#[$meta])* $variant = $val),*
        }

        impl Kind {
            pub const fn from_u8(val: u8) -> Option<Self> {
                match val {
                    $( $val => Some(Self::$variant)),*
                    _ => None,
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ErrorKind {
            $( $(#[$meta])* $variant( $sub_type)),*
        }

        impl ErrorKind {
            pub const fn kind(&self) -> Kind {
                match self {
                    $( ErrorKind::$variant(_) => Kind::$variant),*
                }
            }

            // u16 エンコード
            pub const fn to_u16(self) -> u16 {
                let parent = self.kind() as u8;
                let child = match self {
                    $( ErrorKind::$variant(sub) => sub.to_u8() ),*
                };
                ((parent as u16) << 8) | (child as u16)
            }

            // u16 デコード
            pub const fn from_u16(code: u16) -> Option<Self> {
                let parent_byte = (code >> 8) as u8;
                let child = (code & 0xFF) as u8;

                // kind復元
                let kind = match Kind::from_u8(parent_byte) {
                    Some(k) => k,
                    None => return None,
                };

                // kindからdetailの復元
                match kind {
                    $( Kind::$variant => match $sub_type::from_u8(child) {
                        Some(sub) => Some(ErrorKind::$variant(sub)),
                        None => None,
                    },)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_error_detail {
    ( $(#[$type_meta:meta])* $name:ident { $( $(#[$meta:meta])* $variant:ident = $val:literal ), *$(,)? } ) => {
        $(#[$type_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum $name {
            $( $(#[$meta])* $variant = $val),*
        }

        impl $name {
            pub const fn to_u8(self) -> u8 {
                self as u8
            }

            pub const fn from_u8(val: u8) -> Option<Self> {
                match val {
                    $( $val => Some(Self::$variant)),*
                    _ => None,
                }
            }
        }
    };
}