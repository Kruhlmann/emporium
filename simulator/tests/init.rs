use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[ctor::ctor]
pub fn init_tracing() {
    let (chrome_layer, guard) = ChromeLayerBuilder::new()
        .include_args(true)
        .include_locations(true)
        .build();
    tracing_subscriber::registry()
        .with(chrome_layer)
        .try_init()
        .unwrap();
    let _static_guard: &'static _ = Box::leak(Box::new(guard));
}
