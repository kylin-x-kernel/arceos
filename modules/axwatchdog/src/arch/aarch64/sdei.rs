// =============================================================================
// SDEI-based NMI Source
// =============================================================================

pub mod sdei_nmi {
    //! SDEI-based NMI implementation.

    use aarch64_sdei::{SDEI_EVENT_SOFTWARE_NMI, Sdei};

    use super::*;

    /// SDEI-based NMI source.
    pub struct SdeiNmi {
        sdei: Sdei,
        storage: NmiHandlerStorage,
    }

    impl SdeiNmi {
        /// Create a new SDEI NMI source.
        pub const fn new() -> Self {
            todo!()
        }

        /// Get the underlying SDEI interface.
        pub fn sdei(&self) -> &Sdei {
            todo!()
        }
    }

    impl NmiSource for SdeiNmi {
        fn init(&self) -> NmiResult<()> {
            todo!()
        }

        fn enable(&self) -> NmiResult<()> {
            todo!()
        }

        fn disable(&self) -> NmiResult<()> {
            todo!()
        }

        fn is_enabled(&self) -> bool {
            todo!()
        }

        fn name(&self) -> &'static str {
            "SDEI"
        }
    }

    impl Default for SdeiNmi {
        fn default() -> Self {
            Self::new()
        }
    }
}