use bevy::prelude::App;

#[allow(unused)]
mod input_impl;

pub struct InputSystem {
    inner: input_impl::InputSystem,
}

impl InputSystem {
    pub fn init() -> Result<Self, String> {
        Ok(Self {
            inner: input_impl::InputSystem::init()?,
        })
    }

    pub fn handle_events(&mut self, app: &mut App) {
        self.inner.handle_event(app);
    }
}
