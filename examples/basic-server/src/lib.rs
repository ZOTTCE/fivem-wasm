use fivem::{ref_funcs::RefFunction, server::events::PlayerConnecting};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

async fn handle_connections() {
    let mut events = fivem::server::events::player_connecting();

    while let Some(event) = events.next().await {
        fivem::log(format!(
            "A new player connected: {}",
            event.payload().player_name
        ));

        let _ = fivem::runtime::spawn(show_something(event.into_inner()));
    }
}

async fn show_something(event: PlayerConnecting) {
    event.deferrals.defer.invoke::<(), ()>(());

    fivem::runtime::sleep_for(std::time::Duration::from_millis(10)).await;

    #[derive(Serialize)]
    struct UpdateMessage(String);

    #[derive(Serialize)]
    struct DoneMessage(String);

    let udp_msg = UpdateMessage(String::from("Hello from Rust! Wait 5 seconds, please ..."));

    event.deferrals.update.invoke::<(), _>(vec![udp_msg]);
    fivem::runtime::sleep_for(std::time::Duration::from_secs(5)).await;
    event.deferrals.done.invoke::<(), Vec<DoneMessage>>(vec![]);

    // reject a connection
    // let done_msg = DoneMessage(String::from("do not enter!!"));
    // event.deferrals.done.invoke::<(), _>(vec![done_msg]);
}

fn print_my_keys() {
    println!("START FINDING KEYS:");

    if let Ok(handle) = fivem::server::natives::start_find_kvp("my:") {
        while let Ok(key) = fivem::server::natives::find_kvp(handle) {
            println!("found a new key: {:?}", key);
        }

        let _ = fivem::server::natives::end_find_kvp(handle);
    }

    println!("DONE FINDING KEYS");
}

fn create_export() {
    #[derive(Debug, Deserialize)]
    struct Vector {
        x: f32,
        y: f32,
        z: f32,
    }

    let export = RefFunction::new(|vector: Vec<Vector>| {
        if let Some(vec) = vector.get(0) {
            let length = (vec.x.powi(2) + vec.y.powi(2) + vec.z.powi(2)).sqrt();
            return vec![length];
        }

        vec![0.0]
    });

    fivem::exports::make_export("vecLength", export);
}

async fn test_exports() {
    #[derive(Serialize, Deserialize)]
    struct SomeObject(u32, f32, String);

    // exports("testique", (a, b, c) => console.log(`int: ${a} float: ${b} str: ${c}));
    let testique = fivem::exports::import_function("emitjs", "testique").unwrap();
    testique.invoke::<(), _>(SomeObject(5123, 10.5, String::from("hellow!")));
}

#[no_mangle]
pub extern "C" fn _start() {
    // cleanup prev
    fivem::server::natives::delete_resource_kvp("my:int");
    fivem::server::natives::delete_resource_kvp("my:str");
    fivem::server::natives::delete_resource_kvp("my:float");

    println!("BEFORE:");

    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_int("my:int")
    );
    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_string("my:str")
    );
    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_float("my:float")
    );

    fivem::server::natives::set_resource_kvp("my:str", "stringify");
    fivem::server::natives::set_resource_kvp_float("my:float", 1345.5);
    fivem::server::natives::set_resource_kvp_int("my:int", 55561);

    println!("AFTER:");

    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_int("my:int")
    );
    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_string("my:str")
    );
    println!(
        "{:?}",
        fivem::server::natives::get_resource_kvp_float("my:float")
    );

    print_my_keys();
    create_export();

    let task = test_exports();
    let _ = fivem::runtime::spawn(task);
    let task = handle_connections();
    let _ = fivem::runtime::spawn(task);
}
