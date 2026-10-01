static NIGHTFARER_LOCKED: AtomicBool = AtomicBool::new(false);
static GRANTED_NIGHTFARERS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

const NIGHTFARER_FLAGS: [u32; 10] = [6030, 6031, 6032, 6033, 6034, 6035, 6036, 6037, 6038, 6039];
