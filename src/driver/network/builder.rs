use crate::driver::network::driver::{
    Args,
    Driver,
    NetworkDriverOnXCallable,
    PrivilegeLevel,
};
use crate::driver::GenericDriverBuilder;

/// `Builder` is a struct that holds a bunch of settings/defaults that can be used to build a
/// *network* Driver object -- you must also provide the *generic* driver builder as the network
/// one sits "on top" of that!
pub struct Builder {
    generic_driver_builder: GenericDriverBuilder,
    args: Args,
}

#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::return_self_not_must_use)]
#[allow(clippy::must_use_candidate)]
impl Builder {
    /// Return a new instance of `Builder` with sane defaults set.
    pub fn new(generic_driver_builder: GenericDriverBuilder) -> Self {
        Self {
            generic_driver_builder,
            args: Args::default(),
        }
    }

    /// Sets the `secondary_password` password to use for (enable/escalate) authentication.
    pub fn secondary_password(
        mut self,
        s: &str,
    ) -> Self {
        self.args.secondary_password = s.to_owned();

        self
    }

    /// Sets the privilege levels for the network driver.
    pub fn privilege_levels(
        mut self,
        p: Vec<PrivilegeLevel>,
    ) -> Self {
        self.args.privilege_levels = p;

        self
    }

    /// Sets the `default_desired_privilege_level` for the network driver.
    pub fn default_desired_privilege_level(
        mut self,
        s: &str,
    ) -> Self {
        self.args.default_desired_privilege_level = s.to_owned();

        self
    }

    /// Sets the `on_open` argument of a driver.
    pub fn on_open(
        mut self,
        f: NetworkDriverOnXCallable,
    ) -> Self {
        self.args.on_open = Some(f);

        self
    }

    /// Sets the `on_close` argument of a driver.
    pub fn on_close(
        mut self,
        f: NetworkDriverOnXCallable,
    ) -> Self {
        self.args.on_close = Some(f);

        self
    }

    /// Build "builds" and returns a Driver object.
    #[must_use]
    pub fn build(self) -> Driver {
        Driver::new(self.generic_driver_builder.build(), self.args)
    }
}
