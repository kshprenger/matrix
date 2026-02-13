// There are a lot of Rc small allocations, so we optimize this too using different allocator
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;
