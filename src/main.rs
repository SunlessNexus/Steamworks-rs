use steamworks::interface::Interface;

fn main() {
    let ctx = steamworks::init_gameserver("./libs/linux64/libsteam_api.so".into(),
        480,
        vec![steamworks::steamuser::v23::ISteamUser::VERSION],
        std::net::Ipv4Addr::new(127, 0, 0, 1).into(),
        27015,
        27016,
        steamworks::EServerMode::Authentication,
        "0.0.0.0"
    ).unwrap();
    //let _ = steamworks::init("./libs/win64/steam_api64.dll".into()).unwrap();

    match ctx.create_interface::<steamworks::steamgameserver::v15::ISteamGameServer>() {
        Some(iface) => {
            iface.log_on_anonymous();
            std::thread::sleep(std::time::Duration::from_secs(10));
            println!("Created gameserver steamid {:3}", iface.get_steam_id());
        },
        None => { 
            println!("Failed to create ISteamGameServer interface!");
        }
    };

    let user = match ctx.create_interface::<steamworks::steamuser::v23::ISteamUser>() {
        Some(iface) => iface,
        None => { 
            println!("Failed to create ISteamUser interface!");
            return;
        }
    };
    println!("[SW] Logged as user: {:3}", user.get_steam_id());
}