#[derive(Debug)]
pub struct RouteInfo {
    pub dst:        String,
    pub gw:         String,
    pub mask:       u32,
    pub flags:      String,
    pub metric:     u32,
    pub ref_:       String,
    pub use_:       String,
    pub dev:        String,
}

impl RouteInfo {
    pub fn empty() -> Self {
        Self {
            dst:        String::new(),
            gw:         String::new(),
            mask:       32,
            flags:      String::new(),
            metric:     0,
            ref_:       String::new(),
            use_:       String::new(),
            dev:        String::new(),
        }
    }
}
