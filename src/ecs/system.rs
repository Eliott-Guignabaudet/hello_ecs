pub trait StartupSystem {
    fn startup();
}
pub trait UpdateSystem {
    fn update();
}

pub trait DrawSystem {
    fn draw();
}

pub trait CleanupSystem {
    fn cleanup();
}