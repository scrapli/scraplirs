#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::complexity)]
#![deny(clippy::correctness)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![deny(missing_docs)]
#![warn(clippy::multiple_crate_versions)]
// restriction is wild, but some good things for consistency in there, rather would allow things
// explicitly so any new lints pop up and annoy if they get added and then can decide to keep or
// ditch them!
#![warn(clippy::restriction)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::separated_literal_suffix)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::self_named_module_files)]
#![allow(clippy::multiple_inherent_impl)]
#![allow(clippy::partial_pub_fields)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::multiple_unsafe_ops_per_block)]
#![allow(clippy::single_char_lifetime_names)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::as_conversions)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::unwrap_in_result)]
#![allow(clippy::pub_use)]
#![allow(clippy::arithmetic_side_effects)]

//! scraplirs is a rust implementation of the "scrapli"/"scrapligo" python/go libraries.

/// Channel is the object that consumes from and writes to scraplirs transports. The channel should
/// generally only be interacted with by drivers.
pub mod channel;

/// Scraplirs "drivers" are the primary object users work with.
pub mod driver {
    /// Generic driver is a driver that has no concept of "network" device things -- generic drivers
    /// can be used like a dumb expect type interface for linux or similar devices.
    pub mod generic {
        /// The generic driver builder package,  ya know, for building generic driver stuff.
        pub mod builder;

        /// The actual driver package itself.
        pub mod driver;
    }

    /// The generic driver builder re-exported for convenience.
    pub use crate::driver::generic::builder::Builder as GenericDriverBuilder;

    /// The generic driver re-exported for convenience.
    #[allow(clippy::module_name_repetitions)]
    pub use crate::driver::generic::driver::Driver as GenericDriver;

    /// The generic driver operation options re-exported for convenience.
    pub use crate::driver::generic::driver::OperationOptions as GenericDriverOperationOptions;

    /// Network driver is a driver that wraps `GenericDriver` and adds "network" things like a basic
    /// understanding of privilege levels.
    pub mod network {
        /// The network driver builder package,  ya know, for building network driver stuff.
        pub mod builder;

        /// The actual driver package itself.
        pub mod driver;
    }

    /// The network driver builder re-exported for convenience.
    pub use crate::driver::network::builder::Builder as NetworkDriverBuilder;

    /// The network driver re-exported for convenience.
    #[allow(clippy::module_name_repetitions)]
    pub use crate::driver::network::driver::Driver as NetworkDriver;
}

/// Scraplirs errors.
pub mod errors;

/// Module responsible for dealing with "platform" things -- meaning taking a yaml platform
/// definition and generating a valid scraplirs `GenericDriver` or `NetworkDriver` object.
pub mod platform;

/// Module containing the scraplirs "response" objects -- that is, objects that are returned from
/// successful driver operations.
pub mod response;

/// Transport module holds the base transport and any transport implementations.
pub mod transport {
    /// Base transport module providing trait that all transports must implement.
    pub mod base;

    /// The "system" (/bin/ssh wrapper -- the "original") scrapli transport implementation.
    pub mod system;
}

/// Scraplirs utilities.
pub mod util {
    /// Simple bytes helper functions.
    pub(crate) mod bytes;

    /// Some string helpers.
    pub(crate) mod strings;

    /// A simple queue implementation used in the scraplirs channel.
    pub(crate) mod queue;

    /// Vendor'd ptyprocess form rexpect with extra love for non blocking fd.
    pub(crate) mod ptyprocess;
}
