use futures::StreamExt;

async fn handle_connections() {
    let mut events = fivem::server::events::player_connecting();

    while let Some(event) = events.next().await {
        fivem::log(format!(
            "A new player connected: {}",
            event.payload().player_name
        ));
    }
}

mod kvp {
    use fivem::invoker::{invoke, Val};

    pub fn delete_resource_kvp(key: &str) {
        let _ = invoke::<(), _>(0x7389B5DF, &[Val::String(key)]);
    }

    pub fn end_find_kvp(handle: u32) {
        let _ = invoke::<(), _>(0xB3210203, &[Val::Integer(handle)]);
    }
    pub fn find_kvp(handle: u32) -> Option<String> {
        invoke(0xBD7BEBC5, &[Val::Integer(handle)]).ok()
    }

    pub fn resource_kvp_float(key: &str) -> Option<f32> {
        invoke(0x35BDCEEA, &[Val::String(key)]).ok()
    }

    pub fn resource_kvp_int(key: &str) -> Option<u32> {
        invoke(0x557B586A, &[Val::String(key)]).ok()
    }

    pub fn resource_kvp_string(key: &str) -> Option<String> {
        invoke(0x5240DA5A, &[Val::String(key)]).ok()
    }

    pub fn set_resource_kvp_float(key: &str, val: f32) {
        let _ = invoke::<(), _>(0x9ADD2938, &[Val::String(key), Val::Float(val)]);
    }

    pub fn set_resource_kvp_int(key: &str, val: u32) {
        let _ = invoke::<(), _>(0x6A2B1E8, &[Val::String(key), Val::Integer(val)]);
    }

    pub fn set_resource_kvp(key: &str, val: &str) {
        let _ = invoke::<(), _>(0x21C7A35B, &[Val::String(key), Val::String(val)]);
    }

    pub fn start_find_kvp(prefix: &str) -> Option<u32> {
        invoke(0xDD379006, &[Val::String(prefix)]).ok()
    }
}

fn print_my_keys() {
    println!("START FINDING KEYS:");

    if let Some(handle) = kvp::start_find_kvp("my:") {
        while let Some(key) = kvp::find_kvp(handle) {
            println!("found a new key: {:?}", key);
        }

        kvp::end_find_kvp(handle);
    }

    println!("DONE FINDING KEYS");
}

#[no_mangle]
pub extern "C" fn _start() {
    // cleanup prev
    kvp::delete_resource_kvp("my:int");
    kvp::delete_resource_kvp("my:str");
    kvp::delete_resource_kvp("my:float");

    println!("BEFORE:");

    println!("{:?}", kvp::resource_kvp_int("my:int"));
    println!("{:?}", kvp::resource_kvp_string("my:str"));
    println!("{:?}", kvp::resource_kvp_float("my:float"));

    kvp::set_resource_kvp("my:str", "stringify");
    kvp::set_resource_kvp_float("my:float", 1345.5);
    kvp::set_resource_kvp_int("my:int", 55561);

    println!("AFTER:");

    println!("{:?}", kvp::resource_kvp_int("my:int"));
    println!("{:?}", kvp::resource_kvp_string("my:str"));
    println!("{:?}", kvp::resource_kvp_float("my:float"));

    print_my_keys();

    let task = handle_connections();
    let _ = fivem::runtime::spawn(task);
}

/*

    output:

        BEFORE:
        Some(0)
        None
        Some(0.0)
        AFTER:
        Some(55561)
        Some("stringify")
        Some(1345.5)
        START FINDING KEYS:
        found a new key: "my:float"
        found a new key: "my:int"
        found a new key: "my:str"
        DONE FINDING KEYS

*/
