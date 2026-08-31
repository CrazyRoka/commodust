use system::System;

fn main() {
    let mut system = System::new();

    for _ in 0..10000 {
        system.tick();
    }
}
