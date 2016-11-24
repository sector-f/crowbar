extern crate cairo;

extern crate clap;
use clap::{App, Arg};

extern crate xcb;

extern crate xcb_util as xcbu;
use xcbu::{icccm, ewmh};

fn is_valid_size(n: String) -> Result<(), String> {
    let size = n.split('x').flat_map(str::parse::<u16>).collect::<Vec<_>>();
    if size.len() != 2 {
        return Err("Invalid size specified".to_string());
    }
    Ok(())
}

fn is_valid_offset(n: String) -> Result<(), String> {
    match n.parse::<i16>() {
        Ok(_) => return Ok(()),
        Err(e) => return Err(format!("{}", e)),
    }
}

fn main() {
    let matches = App::new("crowbar")
        .version("0.1.0")
        .about("The most-useful bar")
        .arg(Arg::with_name("size")
             .short("s")
             .long("size")
             .value_name("WIDTHxHEIGHT")
             .help("Sets the bar dimensions")
             .validator(is_valid_size)
             .takes_value(true))
        .arg(Arg::with_name("x")
             .short("x")
             .long("x-offset")
             .value_name("OFFSET")
             .help("Sets the bar's X offset")
             .validator(is_valid_offset)
             .takes_value(true))
        .arg(Arg::with_name("y")
             .short("y")
             .long("y-offset")
             .value_name("OFFSET")
             .help("Sets the bar's Y offset")
             .validator(is_valid_offset)
             .takes_value(true))
        .get_matches();

    let (connection, screen) =
        xcb::Connection::connect(None).expect("Failed to get connection to X server");
    let connection = ewmh::Connection::connect(connection).map_err(|(e, _)| e).unwrap();
    let screen = connection.get_setup().roots().nth(screen as usize).unwrap();

    let (w, h) = match matches.value_of("size") {
        Some(size) => {
            let sizes = &size.split('x').collect::<Vec<_>>();
            let w = sizes[0].parse::<u16>().unwrap();
            let h = sizes[1].parse::<u16>().unwrap();
            (w, h)
        },
        None => {
            (screen.width_in_pixels(), 30)
        },
    };
    let x = matches.value_of("x").map_or(0, |n| n.parse::<i16>().unwrap());
    let y = matches.value_of("y").map_or(0, |n| n.parse::<i16>().unwrap());


    let wid = connection.generate_id();
    xcb::create_window(
        &connection,
        xcb::COPY_FROM_PARENT as u8,
        wid,
        screen.root(),
        x,
        y,
        w,
        h,
        10, // border_width
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &[(xcb::CW_BACKING_PIXEL, screen.black_pixel())],
    );

    icccm::set_wm_class(&connection, wid, "crowbar", "Bar");
    icccm::set_wm_name(&connection, wid, "crowbar");
    ewmh::set_wm_name(&connection, wid, "crowbar");
    ewmh::set_wm_state(&connection, wid, &[connection.WM_STATE_STICKY(), connection.WM_STATE_ABOVE()]);
    ewmh::set_wm_window_type(&connection, wid, &[connection.WM_WINDOW_TYPE_DOCK()]);

    xcb::map_window(&connection, wid);
    connection.flush();

    loop {}
}
