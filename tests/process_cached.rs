use lunatic::{serializer::Bincode, spawn_link, test};
use lunatic_cached_process::{cached_process, CachedLookup};
use serde::{Deserialize, Serialize};

const PROCESS_NAME: &str = "my-awesome-process";

cached_process! {
    static FOO: Process<Message> = PROCESS_NAME;
    static BAR: Process<Message, Bincode> = PROCESS_NAME;
    static BAZ: ProcessRef<Message> = PROCESS_NAME;
}

#[derive(Serialize, Deserialize)]
enum Message {
    Hi,
}

#[test]
fn lookup() {
    assert!(FOO.get().is_none());
    assert!(FOO.get().is_none());
    FOO.reset();

    let process = spawn_link!(|mailbox: Mailbox<Message>| {
        #[allow(unreachable_code)]
        loop {
            let _ = mailbox.receive();
        }
    });
    process.register(PROCESS_NAME);

    assert!(FOO.get().is_some());

    process.kill();

    assert!(FOO.get().is_some()); // Should still be some since its cached
}
