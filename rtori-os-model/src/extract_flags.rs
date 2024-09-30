use bitflags::bitflags;

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExtractFlags: u16 {
        const POSITION = 0b0000_0000_0000_0001;
        const ERROR = 0b0000_0000_0000_0010;
        const VELOCITY = 0b0000_0000_0000_0100;
        const FOLD_ANGLE = 0b0000_0000_0000_1000;

        const ALL = Self::POSITION.bits() | Self::ERROR.bits() | Self::VELOCITY.bits() | Self::FOLD_ANGLE.bits();
    }
}
